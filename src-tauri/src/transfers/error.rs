use sn_dbc::Error as DbcError;

use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Error type returned by the API
#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum Error {
    /// Not enough balance to perform a transaction
    #[error("Not enough balance: {0}")]
    NotEnoughBalance(String),
    /// An error from the `sn_dbc` crate.
    #[error("Dbc error: {0}")]
    Dbcs(#[from] Box<DbcError>),
    /// DbcReissueFailed
    #[error("DbcReissueFailed: {0}")]
    DbcReissueFailed(String),
}
