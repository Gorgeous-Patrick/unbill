#[derive(Clone, Debug)]
pub enum ServiceEvent {
    LedgerUpdated {
        ledger_id: String,
    },
    PeerConnected {
        ledger_id: String,
        peer: String,
    },
    PeerDisconnected {
        ledger_id: String,
        peer: String,
    },
    SyncError {
        ledger_id: String,
        peer: String,
        error: String,
    },
}
