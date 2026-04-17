use autosurgeon::{Hydrate, Reconcile};

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Ledger {
    pub ledger_id: String,
    pub schema_version: u32,
    pub name: String,
    pub currency: String,
    pub created_at: i64,
    pub members: Vec<Member>,
    pub bills: Vec<super::bill::Bill>,
    pub invitations: Vec<Invitation>,
}

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Member {
    pub user_id: String,
    pub display_name: String,
    pub devices: Vec<Device>,
    pub added_at: i64,
    pub added_by: String,
    pub removed: bool,
}

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Device {
    pub node_id: String,
    pub label: String,
    pub added_at: i64,
}

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Invitation {
    pub token: String,
    pub created_by_user_id: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub used_by_node_id: Option<String>,
}

/// Lightweight summary for list views (no CRDT bytes needed).
#[derive(Clone, Debug)]
pub struct LedgerMeta {
    pub ledger_id: String,
    pub name: String,
    pub currency: String,
    pub created_at: i64,
    pub updated_at: i64,
}
