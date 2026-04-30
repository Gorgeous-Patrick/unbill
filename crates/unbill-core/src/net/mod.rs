// P2P networking via Iroh.
// See net/DESIGN.md before implementing.

mod endpoint;
mod join;
mod protocol;
mod sync;

pub use endpoint::UnbillEndpoint;
pub use join::{run_join_host, run_join_requester};
pub(crate) use protocol::JoinRequest;
pub use sync::run_sync_session;
