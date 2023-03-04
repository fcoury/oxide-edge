use std::error::Error;

use bson::{bson, Bson};

use super::CommandResult;

pub fn run(e: Box<dyn Error>) -> CommandResult {
    let bson = bson! [{
        "ok": Bson::Double(0.0),
        "errmsg": Bson::String(format!("{e}")),
        "code": Bson::Int32(59),
        "codeName": "CommandNotFound",
    }];
    let doc = bson.as_document().unwrap().clone();
    Ok(doc)
}
