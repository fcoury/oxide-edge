use bson::Document;
use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::OpCode;
use r2d2::PooledConnection;
use tracing::{instrument, trace};

use crate::{command::Command, error::Error};

use super::{OpQueryReply, OpReply};

#[derive(Debug)]
pub struct OpQuery(pub mongodb_wire_protocol_parser::OpQuery);

impl OpQuery {
    #[instrument(name = "OpQuery.handle", skip(self))]
    pub async fn handle(
        self,
        db_conn: PooledConnection<DuckdbConnectionManager>,
    ) -> Result<(OpReply, Vec<u8>), Error> {
        let doc = self.run(db_conn).await?;
        let reply = self.reply(doc)?;

        let data: Vec<u8> = reply.clone().into();
        Ok((OpReply::OpQuery(reply), data))
    }

    pub fn reply(self, doc: Document) -> Result<OpQueryReply, Error> {
        let mut reply: OpQueryReply = self.0.into();
        reply.add_document(&doc);
        reply.number_returned = 1;

        Ok(reply)
    }

    async fn run(
        &self,
        db_conn: PooledConnection<DuckdbConnectionManager>,
    ) -> Result<Document, Error> {
        let command = match self.0.query.keys().next() {
            Some(key) => key,
            None => "ismaster",
        };

        trace!("OP_QUERY command: {}", command.to_ascii_lowercase());
        let msg = OpCode::OpQuery(self.0.clone());
        Command::run(command.to_string(), db_conn, msg)
    }
}
