// In-memory LedgerStore implementation for unit tests.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use automerge::ChangeHash;

use crate::model::LedgerMeta;

use super::traits::{LedgerStore, LoadedBytes, Result};

#[derive(Default)]
pub struct InMemoryStore {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    ledgers: HashMap<String, StoredLedger>,
    device_meta: HashMap<String, Vec<u8>>,
}

struct StoredLedger {
    meta: LedgerMeta,
    snapshot: Vec<u8>,
    incrementals: Vec<Vec<u8>>,
}

#[async_trait]
impl LedgerStore for InMemoryStore {
    async fn save_ledger_meta(&self, meta: &LedgerMeta) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        let id = meta.ledger_id.to_string();
        inner
            .ledgers
            .entry(id)
            .and_modify(|s| s.meta = meta.clone())
            .or_insert_with(|| StoredLedger {
                meta: meta.clone(),
                snapshot: vec![],
                incrementals: vec![],
            });
        Ok(())
    }

    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        let inner = self.inner.lock().unwrap();
        Ok(inner.ledgers.values().map(|s| s.meta.clone()).collect())
    }

    async fn load_ledger_bytes(&self, ledger_id: &str) -> Result<LoadedBytes> {
        let inner = self.inner.lock().unwrap();
        let stored = inner
            .ledgers
            .get(ledger_id)
            .ok_or_else(|| crate::error::StorageError::Serialization(
                format!("ledger not found: {ledger_id}"),
            ))?;
        Ok(LoadedBytes {
            snapshot: stored.snapshot.clone(),
            incrementals: stored.incrementals.clone(),
        })
    }

    async fn append_incremental(&self, ledger_id: &str, bytes: &[u8]) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(s) = inner.ledgers.get_mut(ledger_id) {
            s.incrementals.push(bytes.to_vec());
        }
        Ok(())
    }

    async fn compact(
        &self,
        ledger_id: &str,
        new_snapshot: &[u8],
        _heads: &[ChangeHash],
    ) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(s) = inner.ledgers.get_mut(ledger_id) {
            s.snapshot = new_snapshot.to_vec();
            s.incrementals.clear();
        }
        Ok(())
    }

    async fn delete_ledger(&self, ledger_id: &str) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.ledgers.remove(ledger_id);
        Ok(())
    }

    async fn load_device_meta(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let inner = self.inner.lock().unwrap();
        Ok(inner.device_meta.get(key).cloned())
    }

    async fn save_device_meta(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        inner.device_meta.insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}
