use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid OP_MSG: {0}")]
    InvalidOpMsg(String),
}
