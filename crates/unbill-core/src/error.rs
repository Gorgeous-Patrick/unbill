use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnbillError {
    #[error("ledger not found: {0}")]
    LedgerNotFound(String),

    #[error("bill not found: {0}")]
    BillNotFound(String),

    #[error("user {0} is not in this ledger")]
    UserNotInLedger(String),

    #[error("user not found: {0}")]
    UserNotFound(String),

    #[error("device not found: {0}")]
    DeviceNotFound(String),

    #[error("invitation invalid or expired")]
    InvalidInvitation,

    #[error("not authorized")]
    NotAuthorized,

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("unauthorized")]
    Unauthorized,

    #[error("http status {0}: {1}")]
    HttpStatus(u16, String),
}

pub type Result<T> = std::result::Result<T, UnbillError>;
