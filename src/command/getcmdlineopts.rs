use bson::{doc, Bson, Document};
use mongodb_wire_protocol_parser::OpCode;

use super::CommandError;

pub fn run(msg: OpCode) -> Result<Document, CommandError> {
    let OpCode::OpMsg(_) = msg else {
        return Err(CommandError::InvalidOpCode);
    };

    Ok(doc! {
        "argv": Bson::Array(vec![Bson::String("oxidedb".to_string())]),
        "parsed": doc!{},
        "ok": Bson::Double(1.into())
    })
}
