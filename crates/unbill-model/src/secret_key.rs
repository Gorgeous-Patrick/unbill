/// The raw 32-byte secret key material for a device's Ed25519 identity.
///
/// Opaque to the model layer. Conversion to network-specific types (e.g.
/// `iroh::SecretKey`) is the responsibility of the network or store crates.
#[derive(Clone)]
pub struct SecretKey([u8; 32]);

impl SecretKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}
