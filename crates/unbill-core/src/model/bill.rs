use autosurgeon::{Hydrate, Reconcile};

use super::amendment::Amendment;

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Bill {
    pub id: String,
    pub payer_user_id: String,
    pub amount_cents: i64,
    pub description: String,
    pub participant_user_ids: Vec<String>,
    pub split_method: SplitMethod,
    pub created_at: i64,
    pub created_by_device: String,
    pub deleted: bool,
    pub amendments: Vec<Amendment>,
}

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub enum SplitMethod {
    Equal,
    Shares(Vec<Share>),
    Exact(Vec<ExactAmount>),
}

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct Share {
    pub user_id: String,
    pub shares: u32,
}

#[derive(Clone, Debug, Reconcile, Hydrate)]
pub struct ExactAmount {
    pub user_id: String,
    pub amount_cents: i64,
}

/// Input type for adding a new bill via the service layer.
#[derive(Clone, Debug)]
pub struct NewBill {
    pub payer_user_id: String,
    pub amount_cents: i64,
    pub description: String,
    pub participant_user_ids: Vec<String>,
    pub split_method: SplitMethod,
}
