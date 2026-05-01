// HTTPS-backed LedgerStore.
// See storage/DESIGN.md §HttpStore for the REST API contract.

use async_trait::async_trait;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::error::StorageError;
use crate::model::{Currency, LedgerMeta, Timestamp, Ulid};

use super::traits::{LedgerStore, Result};

// ---------------------------------------------------------------------------
// JSON shape — kept in sync with FsStore's MetaJson
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

    async fn load_ledger_bytes(&self, ledger_id: &str) -> Result<Vec<u8>> {
        let url = format!("{}/ledgers/{}/snapshot", self.base_url, ledger_id);
        let resp = self.auth(self.client.get(&url)).send().await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }
        let resp = check(resp).await?;
        Ok(resp.bytes().await?.to_vec())
    }

    async fn save_ledger_bytes(&self, ledger_id: &str, bytes: &[u8]) -> Result<()> {
        let url = format!("{}/ledgers/{}/snapshot", self.base_url, ledger_id);
        let resp = self
            .auth(self.client.put(&url))
            .header("Content-Type", "application/octet-stream")
            .body(bytes.to_vec())
            .send()
            .await?;
        check(resp).await?;
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
    use wiremock::{Mock, MockServer, ResponseTemplate};

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

    // --- load_ledger_bytes --------------------------------------------------

    #[tokio::test]
    async fn test_load_ledger_bytes_returns_bytes_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ledgers/00000000000000000000000001/snapshot"))
            .and(bearer_token(API_KEY))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"snapshot".as_ref()))
            .mount(&server)
            .await;

        let bytes = store(&server)
            .load_ledger_bytes("00000000000000000000000001")
            .await
            .unwrap();
        assert_eq!(bytes, b"snapshot");
    }

    #[tokio::test]
    async fn test_load_ledger_bytes_returns_empty_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/ledgers/00000000000000000000000001/snapshot"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let bytes = store(&server)
            .load_ledger_bytes("00000000000000000000000001")
            .await
            .unwrap();
        assert!(bytes.is_empty());
    }

    // --- save_ledger_bytes --------------------------------------------------

    #[tokio::test]
    async fn test_save_ledger_bytes_sends_put_with_octet_stream() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/ledgers/00000000000000000000000001/snapshot"))
            .and(bearer_token(API_KEY))
            .and(header("content-type", "application/octet-stream"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;

        store(&server)
            .save_ledger_bytes("00000000000000000000000001", b"data")
            .await
            .unwrap();
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
