use std::time::{SystemTime, UNIX_EPOCH};

use bson::{doc, Bson};
use mongodb_wire_protocol_parser::OpCode;

use crate::command::{MAX_DOCUMENT_LEN, MAX_MSG_LEN};

use super::CommandResult;

pub fn run(_: OpCode) -> CommandResult {
    let local_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    Ok(doc! {
      "ismaster": Bson::Boolean(true),
      "maxBsonObjectSize": MAX_DOCUMENT_LEN,
      "maxMessageSizeBytes": MAX_MSG_LEN,
      "maxWriteBatchSize": 100000,
      "localTime": Bson::DateTime(bson::DateTime::from_millis(local_time.try_into().unwrap())),
      "minWireVersion": 0,
      "maxWireVersion": 13,
      "readOnly": Bson::Boolean(false),
      "ok": Bson::Double(1.into())
    })
}
