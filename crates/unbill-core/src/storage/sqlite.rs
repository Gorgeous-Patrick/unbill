// SQLite-backed LedgerStore. Implementation begins at M2.
// See storage/DESIGN.md for the schema and snapshot/incremental strategy.

use async_trait::async_trait;
use automerge::ChangeHash;

use crate::model::LedgerMeta;

use super::traits::{LedgerStore, LoadedBytes, Result};

pub struct SqliteStore;

#[async_trait]
impl LedgerStore for SqliteStore {
    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        todo!("M2")
    }

    async fn load_ledger_bytes(&self, _ledger_id: &str) -> Result<LoadedBytes> {
        todo!("M2")
    }

    async fn append_incremental(&self, _ledger_id: &str, _bytes: &[u8]) -> Result<()> {
        todo!("M2")
    }

    async fn compact(
        &self,
        _ledger_id: &str,
        _new_snapshot: &[u8],
        _heads: &[ChangeHash],
    ) -> Result<()> {
        todo!("M2")
    }

    async fn delete_ledger(&self, _ledger_id: &str) -> Result<()> {
        todo!("M2")
    }

    async fn load_sync_state(&self, _ledger_id: &str, _peer: &str) -> Result<Option<Vec<u8>>> {
        todo!("M2")
    }

    async fn save_sync_state(&self, _ledger_id: &str, _peer: &str, _bytes: &[u8]) -> Result<()> {
        todo!("M2")
    }

    async fn load_device_meta(&self, _key: &str) -> Result<Option<Vec<u8>>> {
        todo!("M2")
    }

    async fn save_device_meta(&self, _key: &str, _value: &[u8]) -> Result<()> {
        todo!("M2")
    }
}
