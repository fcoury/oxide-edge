use bson::Document;
use mongodb_wire_protocol_parser::OpCode;
use tracing::{instrument, trace};

use crate::{command::ismaster, error::Error};

use super::{OpQueryReply, OpReply};

#[derive(Debug)]
pub struct OpQuery(pub mongodb_wire_protocol_parser::OpQuery);

impl OpQuery {
    #[instrument(name = "OpQuery.handle", skip(self))]
    pub async fn handle(self) -> Result<(OpReply, Vec<u8>), Box<dyn std::error::Error>> {
        let doc = self.run().await?;
        let reply = self.reply(doc)?;

        let data: Vec<u8> = reply.clone().into();
        Ok((OpReply::OpQuery(reply), data))
    }

    pub fn reply(self, doc: Document) -> Result<OpQueryReply, Box<dyn std::error::Error>> {
        let mut reply: OpQueryReply = self.0.into();
        reply.add_document(&doc);
        reply.number_returned = 1;

        Ok(reply)
    }

    async fn run(&self) -> Result<Document, Error> {
        let command = match self.0.query.keys().next() {
            Some(key) => key,
            None => "ismaster",
        };

        trace!("OP_QUERY command: {}", command.to_ascii_lowercase());
        let msg = OpCode::OpQuery(self.0.clone());
        match command.to_ascii_lowercase().as_ref() {
            "ismaster" => ismaster::run(msg),
            _ => Err(Error::UnknownCommand(command.to_string())),
        }
    }
}
