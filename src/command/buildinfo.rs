use bson::{doc, Bson};
use mongodb_wire_protocol_parser::OpCode;

use crate::{command::MAX_DOCUMENT_LEN, error::Error};

use super::CommandResult;

const MONGO_DB_VERSION: &str = "5.0.42";

pub fn run(msg: OpCode) -> CommandResult {
    let OpCode::OpMsg(_) = msg else {
            return Err(Error::InvalidOpCode);
        };

    Ok(doc! {
        "version": MONGO_DB_VERSION,
        "gitVersion": "30cf72e1380e1732c0e24016f092ed58e38eeb58", // FIXME: get this from git
        "modules": Bson::Array(vec![]),
        "sysInfo": "deprecated",
        "versionArray": Bson::Array(vec![
            Bson::Int32(5),
            Bson::Int32(0),
            Bson::Int32(42),
            Bson::Int32(0),
        ]),
        "bits": Bson::Int32(64),
        "debug": false,
        "maxBsonObjectSize": Bson::Int32(MAX_DOCUMENT_LEN.try_into().unwrap()),
        "buildEnvironment": doc!{},

        // our extensions
        // "ferretdbVersion", version.Get().Version,

        "ok": Bson::Double(1.0)
    })
}
