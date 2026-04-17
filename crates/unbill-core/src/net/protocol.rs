// Handshake, ALPN, message framing. Implementation begins at M4.

pub const ALPN: &[u8] = b"unbill/sync/v1";
pub const PROTOCOL_VERSION: u32 = 1;
