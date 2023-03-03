use std::error::Error;

use bson::{doc, Bson};

use super::CommandResult;

pub fn run(e: Box<dyn Error>) -> CommandResult {
    Ok(doc! {
        "ok": Bson::Double(0.0),
        "errmsg": Bson::String(format!("{}", e)),
        "code": Bson::Int32(59),
        "codeName": "CommandNotFound",
    })
}
