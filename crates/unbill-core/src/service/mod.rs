// Top-level service facade. Implementation begins at M2.
// See DESIGN.md §7 for the full public API.

mod inner;

pub use crate::conflict::ConflictGroup;
pub use crate::settlement::{Settlement, Transaction as SettlementTransaction};
pub use inner::{ServiceEvent, UnbillService};
pub(crate) use inner::{
    load_device_labels, load_pending_invitations, save_device_labels, save_pending_invitations,
};
