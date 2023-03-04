use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid OP_MSG: {0}")]
    InvalidOpMsg(String),
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("unexpected opcode")]
    InvalidOpCode,
    #[error("database error: {0}")]
    Database(duckdb::Error),
    #[error("unexpected error: {0}")]
    Unexpected(Box<dyn std::error::Error>),
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Error::Unexpected(error)
    }
}

impl From<duckdb::Error> for Error {
    fn from(error: duckdb::Error) -> Self {
        Error::Database(error)
    }
}
