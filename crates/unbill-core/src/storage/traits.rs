use async_trait::async_trait;

use crate::error::StorageError;
use crate::model::LedgerMeta;

pub type Result<T> = std::result::Result<T, StorageError>;

pub struct LoadedBytes {
    pub snapshot: Vec<u8>,
    pub incrementals: Vec<Vec<u8>>,
}

#[async_trait]
pub trait LedgerStore: Send + Sync {
    /// Create or update the per-ledger metadata cache.
    /// Must be called before the first `append_incremental` for a new ledger.
    async fn save_ledger_meta(&self, meta: &LedgerMeta) -> Result<()>;
    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>>;
    async fn load_ledger_bytes(&self, ledger_id: &str) -> Result<LoadedBytes>;
    async fn append_incremental(&self, ledger_id: &str, bytes: &[u8]) -> Result<()>;
    async fn compact(
        &self,
        ledger_id: &str,
        new_snapshot: &[u8],
        heads: &[automerge::ChangeHash],
    ) -> Result<()>;
    async fn delete_ledger(&self, ledger_id: &str) -> Result<()>;

    async fn load_device_meta(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn save_device_meta(&self, key: &str, value: &[u8]) -> Result<()>;
}
