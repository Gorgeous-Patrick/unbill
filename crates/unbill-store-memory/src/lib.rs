// In-memory LedgerStore implementation for unit tests.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use unbill_model::{Currency, LedgerMeta, StorageError, Timestamp, Ulid};
use unbill_storage::{LedgerDoc, LedgerStore, StorageResult as Result};

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
    bytes: Vec<u8>,
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
                bytes: vec![],
            });
        Ok(())
    }

    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        let inner = self.inner.lock().unwrap();
        Ok(inner.ledgers.values().map(|s| s.meta.clone()).collect())
    }

    async fn load_ledger(&self, ledger_id: &str) -> Result<Option<LedgerDoc>> {
        let inner = self.inner.lock().unwrap();
        match inner.ledgers.get(ledger_id) {
            None => Ok(None),
            Some(s) if s.bytes.is_empty() => Ok(None),
            Some(s) => LedgerDoc::from_bytes(&s.bytes)
                .map(Some)
                .map_err(|e| StorageError::Serialization(e.to_string())),
        }
    }

    async fn save_ledger(&self, ledger_id: &str, doc: &mut LedgerDoc) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        let bytes = doc.save();
        inner
            .ledgers
            .entry(ledger_id.to_owned())
            .and_modify(|s| s.bytes = bytes.clone())
            .or_insert_with(|| StoredLedger {
                meta: LedgerMeta {
                    ledger_id: Ulid::from_u128(0),
                    name: String::new(),
                    currency: Currency::from_code("USD").unwrap(),
                    created_at: Timestamp::from_millis(0),
                    updated_at: Timestamp::from_millis(0),
                },
                bytes,
            });
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
