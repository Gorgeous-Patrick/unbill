// Typed async client for unbill-server operational endpoints.
// See DESIGN.md for the contract.

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ServerClientError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("service unavailable (device not initialized on server)")]
    ServiceUnavailable,
    #[error("network error: {0}")]
    Network(String),
    #[error("server error {0}: {1}")]
    HttpStatus(u16, String),
}

pub type Result<T> = std::result::Result<T, ServerClientError>;

// ---------------------------------------------------------------------------
// JSON shapes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct InvitationJson {
    url: String,
}

#[derive(Serialize)]
struct JoinBody<'a> {
    url: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<&'a str>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct ServerClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl ServerClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: format!("{}/api/v1", base_url.into().trim_end_matches('/')),
            api_key: api_key.into(),
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req.bearer_auth(&self.api_key)
    }

    /// Trigger an Iroh P2P sync between the server and the given peer node.
    ///
    /// Returns `Ok(())` on 204. Returns `ServiceUnavailable` if the server has
    /// no device key yet.
    pub async fn sync_with_peer(&self, node_id: &str) -> Result<()> {
        let url = format!("{}/peers/{}/sync", self.base_url, node_id);
        let resp = self
            .auth(self.client.post(&url))
            .send()
            .await
            .map_err(|e| ServerClientError::Network(e.to_string()))?;
        check(resp).await?;
        Ok(())
    }

    /// Create a one-time join invitation for the given ledger on the server.
    ///
    /// Returns the `unbill://join/...` URL that the joining device needs.
    /// Returns `NotFound` if the ledger does not exist on the server.
    /// Returns `ServiceUnavailable` if the server has no device key yet.
    pub async fn create_invitation(&self, ledger_id: &str) -> Result<String> {
        let url = format!("{}/ledgers/{}/invitations", self.base_url, ledger_id);
        let resp = self
            .auth(self.client.post(&url))
            .send()
            .await
            .map_err(|e| ServerClientError::Network(e.to_string()))?;
        let resp = check(resp).await?;
        let body: InvitationJson = resp
            .json()
            .await
            .map_err(|e| ServerClientError::Network(e.to_string()))?;
        Ok(body.url)
    }

    /// Join a ledger hosted on another device.
    ///
    /// `invite_url` must be an `unbill://join/...` URL obtained from
    /// `create_invitation` on the host server. `label` is an optional local
    /// display name for the joined ledger.
    ///
    /// Returns `Ok(())` on 204. Returns `ServiceUnavailable` if the server has
    /// no device key yet.
    pub async fn join_ledger(&self, invite_url: &str, label: Option<&str>) -> Result<()> {
        let url = format!("{}/ledgers/join", self.base_url);
        let body = JoinBody {
            url: invite_url,
            label,
        };
        let resp = self
            .auth(self.client.post(&url))
            .json(&body)
            .send()
            .await
            .map_err(|e| ServerClientError::Network(e.to_string()))?;
        check(resp).await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Response helper
// ---------------------------------------------------------------------------

async fn check(resp: reqwest::Response) -> Result<reqwest::Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    match status {
        StatusCode::UNAUTHORIZED => Err(ServerClientError::Unauthorized),
        StatusCode::NOT_FOUND => Err(ServerClientError::NotFound),
        StatusCode::SERVICE_UNAVAILABLE => Err(ServerClientError::ServiceUnavailable),
        _ => {
            let body = resp.text().await.unwrap_or_default();
            Err(ServerClientError::HttpStatus(status.as_u16(), body))
        }
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

    const API_KEY: &str = "test-key";

    fn client(server: &MockServer) -> ServerClient {
        ServerClient::new(server.uri(), API_KEY)
    }

    // --- sync_with_peer ---

    #[tokio::test]
    async fn test_sync_with_peer_sends_post_with_bearer() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/peers/nodexyz/sync"))
            .and(bearer_token(API_KEY))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        client(&server).sync_with_peer("nodexyz").await.unwrap();
    }

    #[tokio::test]
    async fn test_sync_with_peer_returns_service_unavailable_on_503() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/peers/nodexyz/sync"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;
        let err = client(&server).sync_with_peer("nodexyz").await.unwrap_err();
        assert!(matches!(err, ServerClientError::ServiceUnavailable));
    }

    // --- create_invitation ---

    #[tokio::test]
    async fn test_create_invitation_returns_invite_url() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ledgers/LEDGER01/invitations"))
            .and(bearer_token(API_KEY))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_string(r#"{"url":"unbill://join/LEDGER01/node1/token1"}"#)
                    .append_header("content-type", "application/json"),
            )
            .expect(1)
            .mount(&server)
            .await;
        let url = client(&server).create_invitation("LEDGER01").await.unwrap();
        assert_eq!(url, "unbill://join/LEDGER01/node1/token1");
    }

    #[tokio::test]
    async fn test_create_invitation_returns_not_found_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ledgers/MISSING/invitations"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let err = client(&server)
            .create_invitation("MISSING")
            .await
            .unwrap_err();
        assert!(matches!(err, ServerClientError::NotFound));
    }

    #[tokio::test]
    async fn test_create_invitation_returns_service_unavailable_on_503() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ledgers/LEDGER01/invitations"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;
        let err = client(&server)
            .create_invitation("LEDGER01")
            .await
            .unwrap_err();
        assert!(matches!(err, ServerClientError::ServiceUnavailable));
    }

    // --- join_ledger ---

    #[tokio::test]
    async fn test_join_ledger_sends_url_and_label() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ledgers/join"))
            .and(bearer_token(API_KEY))
            .and(header("content-type", "application/json"))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        client(&server)
            .join_ledger("unbill://join/L/node/tok", Some("Groceries"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_join_ledger_without_label() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ledgers/join"))
            .and(bearer_token(API_KEY))
            .respond_with(ResponseTemplate::new(204))
            .expect(1)
            .mount(&server)
            .await;
        client(&server)
            .join_ledger("unbill://join/L/node/tok", None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_join_ledger_returns_service_unavailable_on_503() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/ledgers/join"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;
        let err = client(&server)
            .join_ledger("unbill://join/L/node/tok", None)
            .await
            .unwrap_err();
        assert!(matches!(err, ServerClientError::ServiceUnavailable));
    }

    // --- shared error cases ---

    #[tokio::test]
    async fn test_401_surfaces_as_unauthorized() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/peers/node/sync"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&server)
            .await;
        let err = client(&server).sync_with_peer("node").await.unwrap_err();
        assert!(matches!(err, ServerClientError::Unauthorized));
    }

    #[tokio::test]
    async fn test_500_surfaces_as_http_status_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/peers/node/sync"))
            .respond_with(ResponseTemplate::new(500).set_body_string("oops"))
            .mount(&server)
            .await;
        let err = client(&server).sync_with_peer("node").await.unwrap_err();
        assert!(matches!(err, ServerClientError::HttpStatus(500, _)));
    }
}
