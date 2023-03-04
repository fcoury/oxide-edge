use bson::{doc, Bson};
use mongodb_wire_protocol_parser::OpCode;

use crate::error::Error;

use super::CommandResult;

pub fn run(msg: OpCode) -> CommandResult {
    let OpCode::OpMsg(_) = msg else {
        return Err(Error::InvalidOpCode);
    };

    Ok(doc! {
        "argv": Bson::Array(vec![Bson::String("oxidedb".to_string())]),
        "parsed": doc!{},
        "ok": Bson::Double(1.into())
    })
}
