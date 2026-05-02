use std::fmt;
use std::str::FromStr;

use autosurgeon::{HydrateError, Prop, ReadDoc, Reconciler};

/// The network identity of a device — stored as an opaque string.
///
/// The string representation is the canonical form produced by the network
/// layer (e.g. the hex-encoded Ed25519 public key used by iroh).  The model
/// layer treats it as an opaque identifier; conversion to/from network-layer
/// types is the responsibility of the network crate.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for NodeId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}

/// Construct a deterministic `NodeId` from a single seed byte for use in tests.
#[cfg(any(test, feature = "test-helpers"))]
impl NodeId {
    pub fn from_seed(seed: u8) -> Self {
        Self(format!("{:064x}", seed))
    }
}

// --- autosurgeon integration ---
// Stored as a plain string in the Automerge document.

impl serde::Serialize for NodeId {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for NodeId {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(Self(s))
    }
}

impl autosurgeon::Reconcile for NodeId {
    type Key<'a> = autosurgeon::reconcile::NoKey;

    fn reconcile<R: Reconciler>(&self, reconciler: R) -> Result<(), R::Error> {
        self.0.reconcile(reconciler)
    }
}

impl autosurgeon::Hydrate for NodeId {
    fn hydrate<'a, D: ReadDoc>(
        doc: &'a D,
        obj: &automerge::ObjId,
        prop: Prop<'a>,
    ) -> Result<Self, HydrateError> {
        let s = String::hydrate(doc, obj, prop)?;
        Ok(Self(s))
    }
}
