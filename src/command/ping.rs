use bson::{doc, Bson};

use super::CommandResult;

pub fn run() -> CommandResult {
    Ok(doc! {
      "ok": Bson::Double(1.into())
    })
}
