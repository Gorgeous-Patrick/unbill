// Domain types. See DESIGN.md §4 for the schema and invariants.

mod amendment;
mod bill;
mod member;

pub use amendment::{Amendment, AmendmentSummary, BillAmendment, EffectiveBill};
pub use bill::{Bill, NewBill, Share};
pub use member::{Device, Invitation, Ledger, LedgerMeta, Member};
