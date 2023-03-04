use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid OP_MSG: {0}")]
    InvalidOpMsg(String),
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("unexpected opcode")]
    InvalidOpCode,
}
