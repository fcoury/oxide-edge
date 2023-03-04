use std::error::Error;

use bson::Document;
use mongodb_wire_protocol_parser::OpCode;
use tracing::{debug, instrument};

use crate::command::{ismaster, CommandError};

use super::OpQueryReply;

#[derive(Debug)]
pub struct OpQuery(pub mongodb_wire_protocol_parser::OpQuery);

impl OpQuery {
    #[instrument(name = "OpQuery.handle", skip(self))]
    pub async fn handle(self) -> Result<Vec<u8>, Box<dyn Error>> {
        let doc = self.run().await?;
        let reply = self.reply(doc)?;
        debug!("OpQuery reply={:#?}", reply);

        Ok(reply.into())
    }

    pub fn reply(self, doc: Document) -> Result<OpQueryReply, Box<dyn std::error::Error>> {
        let mut reply: OpQueryReply = self.0.into();
        reply.add_document(&doc);
        reply.number_returned = 1;

        Ok(reply)
    }

    async fn run(&self) -> Result<Document, CommandError> {
        let command = match self.0.query.keys().next() {
            Some(key) => key,
            None => "ismaster",
        };

        debug!("OP_QUERY command: {}", command.to_ascii_lowercase());
        let msg = OpCode::OpQuery(self.0.clone());
        match command.to_ascii_lowercase().as_ref() {
            "ismaster" => ismaster::run(msg),
            _ => Err(CommandError::UnknownCommand(command.to_string())),
        }
    }
}
