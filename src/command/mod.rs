use bson::Document;

use crate::error::Error;

pub mod buildinfo;
pub mod error;
pub mod getcmdlineopts;
pub mod getparameter;
pub mod hello;
pub mod ismaster;
pub mod ping;

pub const MAX_DOCUMENT_LEN: u32 = 16777216;
pub const MAX_MSG_LEN: u32 = 48000000;

pub type CommandResult = Result<Document, Error>;
