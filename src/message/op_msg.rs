use bson::Document;
use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::OpCode;
use r2d2::PooledConnection;
use tracing::{error, instrument, trace};

use crate::{command::Command, error::Error, message::OpMsgReply};

use super::OpReply;

#[derive(Debug)]
pub struct OpMsg(pub mongodb_wire_protocol_parser::OpMsg);

impl OpMsg {
    #[instrument(name = "OpMsg.handle", skip(self))]
    pub async fn handle(
        self,
        db_conn: PooledConnection<DuckdbConnectionManager>,
    ) -> Result<(OpReply, Vec<u8>), Error> {
        let doc = &self.run(db_conn).await?;
        let reply = self.reply(doc)?;

        let data: Vec<u8> = reply.clone().into();

        Ok((OpReply::OpMsg(reply), data))
    }

    pub fn reply(self, doc: &Document) -> Result<OpMsgReply, Error> {
        let mut reply: OpMsgReply = self.0.into();
        reply.add_document(doc).map_err(|e| Error::Unexpected(e))?;

        Ok(reply)
    }

    async fn run(
        &self,
        db_conn: PooledConnection<DuckdbConnectionManager>,
    ) -> Result<Document, Error> {
        if self.0.sections.is_empty() {
            error!("OpMsg must have at least one section");
            return Err(Error::InvalidOpMsg(
                "OpMsg must have at least one section".to_string(),
            ));
        }

        let command = self.0.command();
        trace!("OP_MSG command: {}", command.to_ascii_lowercase());

        let msg = OpCode::OpMsg(self.0.clone());
        Command::run(command, db_conn, msg)
    }
}
