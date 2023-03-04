use bson::Document;
use thiserror::Error;
use tracing::error;

pub mod buildinfo;
pub mod error;
pub mod getcmdlineopts;
pub mod getparameter;
pub mod hello;
pub mod ismaster;
pub mod ping;

pub const MAX_DOCUMENT_LEN: u32 = 16777216;
pub const MAX_MSG_LEN: u32 = 48000000;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("unexpected opcode")]
    InvalidOpCode,
}

pub type CommandResult = Result<Document, CommandError>;
