// Top-level service facade. Implementation begins at M2.
// See DESIGN.md §7 for the full public API.

mod service;

pub use service::{Identity, ServiceEvent, UnbillService};
pub(crate) use service::{
    load_pending_identity_tokens, load_pending_invitations, save_pending_identity_tokens,
    save_pending_invitations,
};
