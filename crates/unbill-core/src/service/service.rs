// UnbillService: top-level facade consumed by CLI and Tauri.
// See DESIGN.md §7 for lifecycle and public API design.

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::error::Result;
use crate::model::{BillAmendment, EffectiveBill, LedgerMeta, Member, NewBill};
use crate::storage::LedgerStore;

pub struct UnbillService {
    pub(crate) store: Arc<dyn LedgerStore>,
    events: broadcast::Sender<ServiceEvent>,
}

#[derive(Clone, Debug)]
pub enum ServiceEvent {
    LedgerUpdated { ledger_id: String },
    PeerConnected { ledger_id: String, peer: String },
    PeerDisconnected { ledger_id: String, peer: String },
    SyncError { ledger_id: String, peer: String, error: String },
}

impl UnbillService {
    pub fn new(store: Arc<dyn LedgerStore>) -> Arc<Self> {
        let (events, _) = broadcast::channel(256);
        Arc::new(Self { store, events })
    }

    // Ledger lifecycle

    pub async fn create_ledger(&self, _name: String, _currency: String) -> Result<String> {
        todo!("M2")
    }

    pub async fn list_ledgers(&self) -> Result<Vec<LedgerMeta>> {
        todo!("M2")
    }

    pub async fn delete_ledger(&self, _ledger_id: &str) -> Result<()> {
        todo!("M2")
    }

    pub async fn export_ledger(&self, _ledger_id: &str) -> Result<Vec<u8>> {
        todo!("M3")
    }

    pub async fn import_ledger(&self, _bytes: &[u8]) -> Result<String> {
        todo!("M3")
    }

    // Bills

    pub async fn add_bill(&self, _ledger_id: &str, _input: NewBill) -> Result<String> {
        todo!("M1")
    }

    pub async fn amend_bill(
        &self,
        _ledger_id: &str,
        _bill_id: &str,
        _input: BillAmendment,
    ) -> Result<()> {
        todo!("M1")
    }

    pub async fn delete_bill(&self, _ledger_id: &str, _bill_id: &str) -> Result<()> {
        todo!("M1")
    }

    pub async fn restore_bill(&self, _ledger_id: &str, _bill_id: &str) -> Result<()> {
        todo!("M1")
    }

    pub async fn list_bills(&self, _ledger_id: &str) -> Result<Vec<EffectiveBill>> {
        todo!("M1")
    }

    // Members

    pub async fn list_members(&self, _ledger_id: &str) -> Result<Vec<Member>> {
        todo!("M2")
    }

    // Settlement

    pub async fn compute_settlement(&self, _ledger_id: &str) -> Result<crate::settlement::Settlement> {
        todo!("M2")
    }

    // Events

    pub fn subscribe(&self) -> broadcast::Receiver<ServiceEvent> {
        self.events.subscribe()
    }
}
