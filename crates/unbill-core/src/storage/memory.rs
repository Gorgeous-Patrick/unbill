// In-memory LedgerStore implementation for unit tests.
// Implementation begins at M2.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use automerge::ChangeHash;

use crate::model::LedgerMeta;

use super::traits::{LedgerStore, LoadedBytes, Result};

#[derive(Default)]
pub struct InMemoryStore {
    _inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    ledgers: HashMap<String, StoredLedger>,
    sync_states: HashMap<(String, String), Vec<u8>>,
    device_meta: HashMap<String, Vec<u8>>,
}

struct StoredLedger {
    meta: LedgerMeta,
    snapshot: Vec<u8>,
    incrementals: Vec<Vec<u8>>,
}

#[async_trait]
impl LedgerStore for InMemoryStore {
    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        let inner = self._inner.lock().unwrap();
        Ok(inner.ledgers.values().map(|s| s.meta.clone()).collect())
    }

    async fn load_ledger_bytes(&self, ledger_id: &str) -> Result<LoadedBytes> {
        let inner = self._inner.lock().unwrap();
        let stored = inner.ledgers.get(ledger_id).ok_or_else(|| {
            super::traits::Result::<()>::Err(crate::error::StorageError::Serialization(format!(
                "ledger not found: {ledger_id}"
            )))
            .unwrap_err()
        })?;
        Ok(LoadedBytes {
            snapshot: stored.snapshot.clone(),
            incrementals: stored.incrementals.clone(),
        })
    }

    async fn append_incremental(&self, ledger_id: &str, bytes: &[u8]) -> Result<()> {
        let mut inner = self._inner.lock().unwrap();
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
        let mut inner = self._inner.lock().unwrap();
        if let Some(s) = inner.ledgers.get_mut(ledger_id) {
            s.snapshot = new_snapshot.to_vec();
            s.incrementals.clear();
        }
        Ok(())
    }

    async fn delete_ledger(&self, ledger_id: &str) -> Result<()> {
        let mut inner = self._inner.lock().unwrap();
        inner.ledgers.remove(ledger_id);
        Ok(())
    }

    async fn load_sync_state(&self, ledger_id: &str, peer: &str) -> Result<Option<Vec<u8>>> {
        let inner = self._inner.lock().unwrap();
        Ok(inner
            .sync_states
            .get(&(ledger_id.to_string(), peer.to_string()))
            .cloned())
    }

    async fn save_sync_state(&self, ledger_id: &str, peer: &str, bytes: &[u8]) -> Result<()> {
        let mut inner = self._inner.lock().unwrap();
        inner
            .sync_states
            .insert((ledger_id.to_string(), peer.to_string()), bytes.to_vec());
        Ok(())
    }

    async fn load_device_meta(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let inner = self._inner.lock().unwrap();
        Ok(inner.device_meta.get(key).cloned())
    }

    async fn save_device_meta(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut inner = self._inner.lock().unwrap();
        inner.device_meta.insert(key.to_string(), value.to_vec());
        Ok(())
    }
}
