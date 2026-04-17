// Persistence layer. See storage/DESIGN.md before implementing.

mod memory;
mod sqlite;
mod traits;

pub use memory::InMemoryStore;
pub use sqlite::SqliteStore;
pub use traits::{LedgerStore, LoadedBytes};
