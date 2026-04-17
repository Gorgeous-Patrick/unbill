// LedgerDoc wraps an Automerge document and exposes typed operations.
// Implementation begins at M1.

use tokio::sync::broadcast;

/// A CRDT-backed in-memory ledger backed by a single Automerge document.
pub struct LedgerDoc {
    _doc: automerge::AutoCommit,
    pub changes: broadcast::Sender<ChangeEvent>,
}

#[derive(Clone, Debug)]
pub enum ChangeEvent {
    LocalWrite,
    RemoteApplied,
}

impl LedgerDoc {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(64);
        Self {
            _doc: automerge::AutoCommit::new(),
            changes: tx,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let doc = automerge::AutoCommit::load(bytes)?;
        let (tx, _) = broadcast::channel(64);
        Ok(Self {
            _doc: doc,
            changes: tx,
        })
    }

    pub fn save(&mut self) -> Vec<u8> {
        self._doc.save()
    }
}

impl Default for LedgerDoc {
    fn default() -> Self {
        Self::new()
    }
}
