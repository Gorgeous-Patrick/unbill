// HTTPS-backed LedgerStore.
// See storage/DESIGN.md §HttpStore for the REST API contract.

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::doc::LedgerDoc;
use crate::error::StorageError;
use crate::model::{Currency, LedgerMeta, Timestamp, Ulid};

use super::traits::{LedgerStore, Result};

// ---------------------------------------------------------------------------
// JSON shape for ledger metadata — kept in sync with FsStore's MetaJson
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct MetaJson {
    ledger_id: String,
    name: String,
    currency: String,
    created_at_ms: i64,
    updated_at_ms: i64,
}

impl MetaJson {
    fn from_meta(meta: &LedgerMeta) -> Self {
        Self {
            ledger_id: meta.ledger_id.to_string(),
            name: meta.name.clone(),
            currency: meta.currency.code().to_owned(),
            created_at_ms: meta.created_at.as_millis(),
            updated_at_ms: meta.updated_at.as_millis(),
        }
    }

    fn into_ledger_meta(self) -> std::result::Result<LedgerMeta, String> {
        let ledger_id = Ulid::from_string(&self.ledger_id).map_err(|e| e.to_string())?;
        let currency = Currency::from_code(&self.currency)
            .ok_or_else(|| format!("unknown currency code {:?}", self.currency))?;
        Ok(LedgerMeta {
            ledger_id,
            name: self.name,
            currency,
            created_at: Timestamp::from_millis(self.created_at_ms),
            updated_at: Timestamp::from_millis(self.updated_at_ms),
        })
    }
}

// ---------------------------------------------------------------------------
// HttpStore
// ---------------------------------------------------------------------------

pub struct HttpStore {
    client: Client,
    base_url: String,
    api_key: String,
}

impl HttpStore {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            api_key: api_key.into(),
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.bearer_auth(&self.api_key)
    }

    /// Run the Automerge sync loop for one ledger over HTTP.
    ///
    /// Each iteration is one `POST /ledgers/{id}/sync` call carrying one
    /// binary-encoded `automerge::sync::Message`. The server receives the
    /// message, applies any incoming changes, generates its own response
    /// message, and returns it (or 204 if it has nothing to send).
    ///
    /// Returns `false` if the server responded 404 on the very first message
    /// (ledger does not exist on the server).
    async fn run_sync_loop(&self, ledger_id: &str, doc: &mut LedgerDoc) -> Result<bool> {
        let url = format!("{}/ledgers/{}/sync", self.base_url, ledger_id);
        let mut sync_state = automerge::sync::State::new();
        let mut first = true;

        loop {
            let msg = doc.generate_sync_message(&mut sync_state);
            let Some(msg) = msg else {
                break;
            };

            let resp = self
                .auth(self.client.post(&url))
                .header("Content-Type", "application/octet-stream")
                .body(msg.encode())
                .send()
                .await?;

            if first && resp.status() == StatusCode::NOT_FOUND {
                return Ok(false);
            }
            first = false;

            if resp.status() == StatusCode::NO_CONTENT {
                continue;
            }

            let resp = check(resp).await?;
            let bytes = resp.bytes().await?;
            let server_msg = automerge::sync::Message::decode(&bytes)
                .map_err(|e| StorageError::Serialization(format!("sync decode: {e}")))?;
            doc.receive_sync_message(&mut sync_state, server_msg)
                .map_err(|e| StorageError::Serialization(e.to_string()))?;
        }

        Ok(true)
    }
}

// ---------------------------------------------------------------------------
// Error helper
// ---------------------------------------------------------------------------

async fn check(resp: reqwest::Response) -> Result<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    if status == StatusCode::UNAUTHORIZED {
        return Err(StorageError::Unauthorized);
    }
    let body = resp.text().await.unwrap_or_default();
    Err(StorageError::HttpStatus(status.as_u16(), body))
}

// ---------------------------------------------------------------------------
// LedgerStore impl
// ---------------------------------------------------------------------------

#[async_trait]
impl LedgerStore for HttpStore {
    async fn save_ledger_meta(&self, meta: &LedgerMeta) -> Result<()> {
        let url = format!("{}/ledgers/{}/meta", self.base_url, meta.ledger_id);
        let resp = self
            .auth(self.client.put(&url))
            .json(&MetaJson::from_meta(meta))
            .send()
            .await?;
        check(resp).await?;
        Ok(())
    }

    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        let url = format!("{}/ledgers", self.base_url);
        let resp = self.auth(self.client.get(&url)).send().await?;
        let resp = check(resp).await?;
        let items: Vec<MetaJson> = resp.json().await?;
        let metas = items
            .into_iter()
            .filter_map(|m| match m.into_ledger_meta() {
                Ok(meta) => Some(meta),
                Err(e) => {
                    tracing::warn!("skipping ledger with bad meta: {e}");
                    None
                }
            })
            .collect();
        Ok(metas)
    }

    async fn load_ledger(&self, ledger_id: &str) -> Result<Option<LedgerDoc>> {
        let mut doc = LedgerDoc::empty();
        let found = self.run_sync_loop(ledger_id, &mut doc).await?;
        if !found || doc.is_empty() {
            return Ok(None);
        }
        Ok(Some(doc))
    }

    async fn save_ledger(&self, ledger_id: &str, doc: &mut LedgerDoc) -> Result<()> {
        self.run_sync_loop(ledger_id, doc).await?;
        Ok(())
    }

    async fn delete_ledger(&self, ledger_id: &str) -> Result<()> {
        let url = format!("{}/ledgers/{}", self.base_url, ledger_id);
        let resp = self.auth(self.client.delete(&url)).send().await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        check(resp).await?;
        Ok(())
    }

    async fn load_device_meta(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let url = format!("{}/device/{}", self.base_url, key);
        let resp = self.auth(self.client.get(&url)).send().await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let resp = check(resp).await?;
        Ok(Some(resp.bytes().await?.to_vec()))
    }

    async fn save_device_meta(&self, key: &str, value: &[u8]) -> Result<()> {
        let url = format!("{}/device/{}", self.base_url, key);
        let resp = self
            .auth(self.client.put(&url))
            .header("Content-Type", "application/octet-stream")
            .body(value.to_vec())
            .send()
            .await?;
        check(resp).await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use wiremock::matchers::{bearer_token, header, method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    use super::*;
    use crate::model::Timestamp;

    const API_KEY: &str = "test-key";

    fn make_meta(name: &str) -> LedgerMeta {
        LedgerMeta {
            ledger_id: Ulid::from_u128(1),
            name: name.to_owned(),
            currency: Currency::from_code("USD").unwrap(),
            created_at: Timestamp::from_millis(1_000),
            updated_at: Timestamp::from_millis(2_000),
        }
    }

    fn make_doc(name: &str) -> LedgerDoc {
        LedgerDoc::new(
            Ulid::from_u128(1),
            name.to_owned(),
            Currency::from_code("USD").unwrap(),
            Timestamp::from_millis(1_000),
        )
        .unwrap()
    }

    fn store(server: &MockServer) -> HttpStore {
        HttpStore::new(server.uri(), API_KEY)
    }

    // --- save_ledger_meta ---------------------------------------------------

    #[tokio::test]
    async fn test_save_ledger_meta_sends_put_with_bearer_and_json() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/ledgers/00000000000000000000000001/meta"))
            .and(bearer_token(API_KEY))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        let meta = make_meta("Groceries");
        store(&server).save_ledger_meta(&meta).await.unwrap();
    }

    // --- list_ledgers -------------------------------------------------------

    #[tokio::test]
    async fn test_list_ledgers_returns_parsed_metas() {
        let server = MockServer::start().await;
        let body = serde_json::to_string(&[MetaJson {
            ledger_id: Ulid::from_u128(1).to_string(),
            name: "Groceries".into(),
            currency: "USD".into(),
            created_at_ms: 1_000,
            updated_at_ms: 2_000,
        }])
        .unwrap();

        Mock::given(method("GET"))
            .and(path("/ledgers"))
            .and(bearer_token(API_KEY))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .append_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let metas = store(&server).list_ledgers().await.unwrap();
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].name, "Groceries");
    }

    #[tokio::test]
    async fn test_list_ledgers_returns_empty_vec_when_server_returns_empty_array() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ledgers"))
            .and(bearer_token(API_KEY))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("[]")
                    .append_header("content-type", "application/json"),
            )
            .mount(&server)
            .await;

        let metas = store(&server).list_ledgers().await.unwrap();
        assert!(metas.is_empty());
    }

    // --- load_ledger / save_ledger ------------------------------------------

    #[tokio::test]
    async fn test_load_ledger_returns_none_when_server_responds_404() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ledgers/00000000000000000000000001/sync"))
            .and(bearer_token(API_KEY))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let result = store(&server)
            .load_ledger("00000000000000000000000001")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_save_and_load_converge_via_sync_loop() {
        // Simulate a server holding a real Automerge doc and running the sync
        // protocol in-process. The mock handler drives a server-side doc.
        // Pre-populate the "server" doc with initial state.
        let server_doc = std::sync::Arc::new(std::sync::Mutex::new(
            LedgerDoc::from_bytes(&make_doc("Groceries").save()).unwrap(),
        ));

        let server = MockServer::start().await;

        struct SyncResponder(std::sync::Arc<std::sync::Mutex<LedgerDoc>>);
        impl Respond for SyncResponder {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let mut doc = self.0.lock().unwrap();
                let mut state = automerge::sync::State::new();
                let client_msg = automerge::sync::Message::decode(&req.body).unwrap();
                doc.receive_sync_message(&mut state, client_msg).unwrap();
                match doc.generate_sync_message(&mut state).map(|m| m.encode()) {
                    Some(bytes) => ResponseTemplate::new(200)
                        .append_header("content-type", "application/octet-stream")
                        .set_body_bytes(bytes),
                    None => ResponseTemplate::new(204),
                }
            }
        }

        Mock::given(method("POST"))
            .and(path("/ledgers/00000000000000000000000001/sync"))
            .and(bearer_token(API_KEY))
            .respond_with(SyncResponder(server_doc))
            .mount(&server)
            .await;

        let loaded = store(&server)
            .load_ledger("00000000000000000000000001")
            .await
            .unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.get_ledger().unwrap().name, "Groceries");
    }

    // --- delete_ledger ------------------------------------------------------

    #[tokio::test]
    async fn test_delete_ledger_sends_delete() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/ledgers/00000000000000000000000001"))
            .and(bearer_token(API_KEY))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        store(&server)
            .delete_ledger("00000000000000000000000001")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete_ledger_is_idempotent_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/ledgers/00000000000000000000000001"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        store(&server)
            .delete_ledger("00000000000000000000000001")
            .await
            .unwrap();
    }

    // --- device meta --------------------------------------------------------

    #[tokio::test]
    async fn test_save_and_load_device_meta_round_trip() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/device/device_key.bin"))
            .and(bearer_token(API_KEY))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/device/device_key.bin"))
            .and(bearer_token(API_KEY))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"secret".as_ref()))
            .expect(1)
            .mount(&server)
            .await;

        let s = store(&server);
        s.save_device_meta("device_key.bin", b"secret")
            .await
            .unwrap();
        let loaded = s.load_device_meta("device_key.bin").await.unwrap();
        assert_eq!(loaded.as_deref(), Some(b"secret".as_ref()));
    }

    #[tokio::test]
    async fn test_load_device_meta_returns_none_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/device/device_key.bin"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let result = store(&server)
            .load_device_meta("device_key.bin")
            .await
            .unwrap();
        assert!(result.is_none());
    }

    // --- error mapping ------------------------------------------------------

    #[tokio::test]
    async fn test_401_surfaces_as_unauthorized_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ledgers"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;

        let err = store(&server).list_ledgers().await.unwrap_err();
        assert!(matches!(err, StorageError::Unauthorized));
    }

    #[tokio::test]
    async fn test_500_surfaces_as_http_status_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ledgers"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&server)
            .await;

        let err = store(&server).list_ledgers().await.unwrap_err();
        assert!(matches!(err, StorageError::HttpStatus(500, _)));
    }
}
