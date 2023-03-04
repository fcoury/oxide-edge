use bson::Document;
use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::OpCode;
use r2d2::PooledConnection;

use crate::error::Error;

pub mod buildinfo;
pub mod error;
pub mod getcmdlineopts;
pub mod getparameter;
pub mod hello;
pub mod insert;
pub mod ismaster;
pub mod listdatabases;
pub mod ping;

pub const MAX_DOCUMENT_LEN: u32 = 16777216;
pub const MAX_MSG_LEN: u32 = 48000000;

pub type CommandResult = Result<Document, Error>;

pub enum Command {
    BuildInfo,
    GetCmdLineOpts,
    GetParameter,
    Hello,
    IsMaster,
    ListDatabases,
    Ping,
    Insert,
}

impl Command {
    pub fn run(
        command: String,
        db_conn: PooledConnection<DuckdbConnectionManager>,
        op_code: OpCode,
    ) -> CommandResult {
        match Command::from_command_str(&command) {
            Some(cmd) => cmd.execute(op_code, db_conn),
            None => error::run(Box::new(Error::UnknownCommand(command))),
        }
    }

    pub fn from_command_str(s: &str) -> Option<Command> {
        match s.to_ascii_lowercase().as_str() {
            "buildinfo" => Some(Command::BuildInfo),
            "getcmdlineopts" => Some(Command::GetCmdLineOpts),
            "getparameter" => Some(Command::GetParameter),
            "hello" => Some(Command::Hello),
            "ismaster" => Some(Command::IsMaster),
            "ping" => Some(Command::Ping),
            "listdatabases" => Some(Command::ListDatabases),
            "insert" => Some(Command::Insert),
            _ => None,
        }
    }

    pub fn execute(
        &self,
        op_code: OpCode,
        db_conn: PooledConnection<DuckdbConnectionManager>,
    ) -> CommandResult {
        match self {
            Command::BuildInfo => buildinfo::run(op_code),
            Command::GetCmdLineOpts => getcmdlineopts::run(op_code),
            Command::GetParameter => getparameter::run(op_code),
            Command::Hello => hello::run(),
            Command::IsMaster => ismaster::run(op_code),
            Command::Ping => ping::run(),
            Command::ListDatabases => listdatabases::run(op_code, db_conn),
            Command::Insert => insert::run(op_code, db_conn),
        }
    }
}
