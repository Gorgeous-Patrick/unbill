// Flat-file LedgerStore backed by `tokio::fs`.
// See DESIGN.md §5 for the directory layout and snapshot/incremental strategy.

use std::path::PathBuf;

use async_trait::async_trait;
use automerge::ChangeHash;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt as _;

use crate::error::StorageError;
use crate::model::{Currency, LedgerMeta, Timestamp, Ulid};

use super::traits::{LedgerStore, LoadedBytes, Result};

pub struct FsStore {
    root: PathBuf,
}

impl FsStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn ledger_dir(&self, ledger_id: &str) -> PathBuf {
        self.root.join("ledgers").join(ledger_id)
    }
}

// ---------------------------------------------------------------------------
// JSON mirror of LedgerMeta — plain primitives, no domain-type imports needed
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
        let ledger_id =
            Ulid::from_string(&self.ledger_id).map_err(|e| e.to_string())?;
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
// LedgerStore impl
// ---------------------------------------------------------------------------

#[async_trait]
impl LedgerStore for FsStore {
    async fn save_ledger_meta(&self, meta: &LedgerMeta) -> Result<()> {
        let dir = self.ledger_dir(&meta.ledger_id.to_string());
        tokio::fs::create_dir_all(&dir).await?;

        let json = serde_json::to_string_pretty(&MetaJson::from_meta(meta))
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        atomic_write(dir.join("meta.json"), json.as_bytes()).await
    }

    async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        let ledgers_dir = self.root.join("ledgers");

        let mut entries = match tokio::fs::read_dir(&ledgers_dir).await {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(e.into()),
        };

        let mut metas = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }
            let meta_path = entry.path().join("meta.json");
            match tokio::fs::read(&meta_path).await {
                Ok(bytes) => {
                    match serde_json::from_slice::<MetaJson>(&bytes)
                        .map_err(|e| e.to_string())
                        .and_then(|m| m.into_ledger_meta())
                    {
                        Ok(meta) => metas.push(meta),
                        Err(e) => {
                            tracing::warn!("skipping {:?}: {e}", entry.path());
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tracing::warn!("skipping {:?}: no meta.json", entry.path());
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(metas)
    }

    async fn load_ledger_bytes(&self, ledger_id: &str) -> Result<LoadedBytes> {
        let dir = self.ledger_dir(ledger_id);

        let snapshot = read_or_empty(dir.join("snapshot.bin")).await?;

        let changes_raw = read_or_empty(dir.join("changes.bin")).await?;
        let incrementals = if changes_raw.is_empty() {
            vec![]
        } else {
            vec![changes_raw]
        };

        Ok(LoadedBytes { snapshot, incrementals })
    }

    async fn append_incremental(&self, ledger_id: &str, bytes: &[u8]) -> Result<()> {
        let path = self.ledger_dir(ledger_id).join("changes.bin");
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;
        file.write_all(bytes).await?;
        file.flush().await?;
        Ok(())
    }

    async fn compact(
        &self,
        ledger_id: &str,
        new_snapshot: &[u8],
        _heads: &[ChangeHash],
    ) -> Result<()> {
        let dir = self.ledger_dir(ledger_id);

        // Crash-safe: write to .tmp then rename over the real file.
        atomic_write(dir.join("snapshot.bin"), new_snapshot).await?;

        // Truncate changes.bin — idempotent if the rename already completed.
        tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(dir.join("changes.bin"))
            .await?;

        Ok(())
    }

    async fn delete_ledger(&self, ledger_id: &str) -> Result<()> {
        match tokio::fs::remove_dir_all(self.ledger_dir(ledger_id)).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    async fn load_device_meta(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match tokio::fs::read(self.root.join(key)).await {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn save_device_meta(&self, key: &str, value: &[u8]) -> Result<()> {
        tokio::fs::create_dir_all(&self.root).await?;
        atomic_write(self.root.join(key), value).await
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write `bytes` to `path` atomically via a `.tmp` sibling + rename.
async fn atomic_write(path: PathBuf, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension("tmp");
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, &path).await?;
    Ok(())
}

/// Read `path` returning its contents, or an empty `Vec` if the file does not
/// exist.
async fn read_or_empty(path: PathBuf) -> Result<Vec<u8>> {
    match tokio::fs::read(&path).await {
        Ok(b) => Ok(b),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
        Err(e) => Err(e.into()),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Timestamp;

    fn make_meta(name: &str) -> LedgerMeta {
        LedgerMeta {
            ledger_id: Ulid::from_u128(1),
            name: name.to_owned(),
            currency: Currency::from_code("USD").unwrap(),
            created_at: Timestamp::from_millis(1_000),
            updated_at: Timestamp::from_millis(2_000),
        }
    }

    #[tokio::test]
    async fn test_save_and_list_ledger_meta() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsStore::new(dir.path().to_path_buf());

        assert!(store.list_ledgers().await.unwrap().is_empty());

        let meta = make_meta("Groceries");
        store.save_ledger_meta(&meta).await.unwrap();

        let listed = store.list_ledgers().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Groceries");
        assert_eq!(listed[0].currency.code(), "USD");
        assert_eq!(listed[0].created_at.as_millis(), 1_000);
        assert_eq!(listed[0].updated_at.as_millis(), 2_000);
    }

    #[tokio::test]
    async fn test_append_incremental_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsStore::new(dir.path().to_path_buf());

        let meta = make_meta("Test");
        store.save_ledger_meta(&meta).await.unwrap();

        let id = meta.ledger_id.to_string();
        store.append_incremental(&id, b"chunk1").await.unwrap();
        store.append_incremental(&id, b"chunk2").await.unwrap();

        let loaded = store.load_ledger_bytes(&id).await.unwrap();
        assert!(loaded.snapshot.is_empty());
        assert_eq!(loaded.incrementals, vec![b"chunk1chunk2".to_vec()]);
    }

    #[tokio::test]
    async fn test_compact_clears_changes() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsStore::new(dir.path().to_path_buf());

        let meta = make_meta("Test");
        store.save_ledger_meta(&meta).await.unwrap();
        let id = meta.ledger_id.to_string();

        store.append_incremental(&id, b"some changes").await.unwrap();
        store.compact(&id, b"full snapshot", &[]).await.unwrap();

        let loaded = store.load_ledger_bytes(&id).await.unwrap();
        assert_eq!(loaded.snapshot, b"full snapshot");
        assert!(loaded.incrementals.is_empty());
    }

    #[tokio::test]
    async fn test_delete_ledger() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsStore::new(dir.path().to_path_buf());

        let meta = make_meta("ToDelete");
        store.save_ledger_meta(&meta).await.unwrap();
        let id = meta.ledger_id.to_string();

        assert_eq!(store.list_ledgers().await.unwrap().len(), 1);
        store.delete_ledger(&id).await.unwrap();
        assert!(store.list_ledgers().await.unwrap().is_empty());

        // Idempotent
        store.delete_ledger(&id).await.unwrap();
    }

    #[tokio::test]
    async fn test_device_meta_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = FsStore::new(dir.path().to_path_buf());

        assert!(store.load_device_meta("device_key.bin").await.unwrap().is_none());

        store.save_device_meta("device_key.bin", b"secret").await.unwrap();
        let loaded = store.load_device_meta("device_key.bin").await.unwrap();
        assert_eq!(loaded.as_deref(), Some(b"secret".as_ref()));
    }
}
