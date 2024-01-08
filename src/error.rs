use candid::{CandidType, Deserialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Deserialize, CandidType, Eq, PartialEq)]
pub enum Error {
    #[error("internal error: {0}")]
    Internal(String),

    #[error("the user has no permission to call this method")]
    NotAuthorized,

    #[error("stable pair not found: {0}")]
    StableError(String),
}
