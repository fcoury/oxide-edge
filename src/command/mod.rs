use bson::Document;
use mongodb_wire_protocol_parser::{OpCode, OpMsg, OpQuery};
use thiserror::Error;
use tracing::{debug, error};

mod buildinfo;
mod error;
mod getcmdlineopts;
mod getparameter;
mod hello;
mod ismaster;
mod ping;

pub const MAX_DOCUMENT_LEN: u32 = 16777216;
pub const MAX_MSG_LEN: u32 = 48000000;
pub const HEADER_SIZE: u32 = 16;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("unexpected opcode")]
    InvalidOpCode,
}

pub type CommandResult = Result<Document, CommandError>;

pub fn run_op_query(op_query: OpQuery) -> Result<Document, CommandError> {
    let query = op_query.clone().query;
    let command = match query.keys().next() {
        Some(key) => key,
        None => "ismaster",
    };

    debug!("OP_QUERY command: {}", command.to_ascii_lowercase());
    let msg = OpCode::OpQuery(op_query);
    match command.to_ascii_lowercase().as_ref() {
        "ismaster" => ismaster::run(msg),
        _ => Err(CommandError::UnknownCommand(command.to_string())),
    }
}

pub fn run_op_msg(op_msg: OpMsg) -> Result<Document, CommandError> {
    let command = op_msg.command();
    debug!("OP_MSG command: {}", command.to_ascii_lowercase());

    let msg = OpCode::OpMsg(op_msg.clone());
    match command.to_ascii_lowercase().as_ref() {
        "ismaster" => ismaster::run(msg),
        "buildinfo" => buildinfo::run(msg),
        "getcmdlineopts" => getcmdlineopts::run(msg),
        "getparameter" => getparameter::run(msg),
        "ping" => ping::run(),
        "hello" => hello::run(),
        _ => {
            error!("unknown command: {}", command);
            error::run(Box::new(CommandError::UnknownCommand(command.to_string())))
        }
    }
}
