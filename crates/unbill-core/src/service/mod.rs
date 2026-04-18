// Top-level service facade. Implementation begins at M2.
// See DESIGN.md §7 for the full public API.

mod inner;

pub(crate) use inner::{
    load_pending_identity_tokens, load_pending_invitations, save_pending_identity_tokens,
    save_pending_invitations,
};
pub use inner::{Identity, ServiceEvent, UnbillService};
