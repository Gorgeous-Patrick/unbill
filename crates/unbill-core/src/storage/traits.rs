use async_trait::async_trait;

use crate::doc::LedgerDoc;
use crate::model::LedgerMeta;

pub type Result<T> = std::result::Result<T, crate::error::StorageError>;

#[async_trait]
pub trait LedgerStore: Send + Sync {
    /// Create or update the per-ledger metadata cache.
    async fn save_ledger_meta(&self, meta: &LedgerMeta) -> Result<()>;
    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>>;

    /// Load a ledger document. Returns `None` if the ledger has never been saved.
    async fn load_ledger(&self, ledger_id: &str) -> Result<Option<LedgerDoc>>;

    /// Persist a ledger document. The store may apply remote changes back into
    /// `doc` before returning (e.g. `HttpStore` merges server-side changes);
    /// callers must treat `doc` as the authoritative merged state after the call.
    async fn save_ledger(&self, ledger_id: &str, doc: &mut LedgerDoc) -> Result<()>;

    async fn delete_ledger(&self, ledger_id: &str) -> Result<()>;

    async fn load_device_meta(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn save_device_meta(&self, key: &str, value: &[u8]) -> Result<()>;
}
