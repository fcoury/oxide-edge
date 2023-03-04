use std::error::Error;

use bson::{ser, Document};
use mongodb_wire_protocol_parser::OpCode;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{debug, instrument};

use crate::{
    command::{ismaster, CommandError, HEADER_SIZE},
    log::log,
};

#[derive(Debug)]
pub struct OpQuery(pub mongodb_wire_protocol_parser::OpQuery);

impl OpQuery {
    #[instrument(name = "OpQuery.handle", skip(self, inbound))]
    pub async fn handle(self, id: &str, inbound: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let doc = self.run().await?;

        let query = self.0;
        let docs = ser::to_vec(&doc)?;
        let message_length = HEADER_SIZE + 20 + docs.len() as u32;

        // header
        inbound.write_all(&message_length.to_le_bytes()).await?;
        inbound.write_all(&0u32.to_le_bytes()).await?; // request_id
        inbound
            .write_all(&query.header.request_id.to_le_bytes())
            .await?; // response_to
        inbound.write_all(&1u32.to_le_bytes()).await?; // opcode - OP_REPLY = 1

        // reply
        inbound.write_all(&query.flags.to_le_bytes()).await?; // flags
        inbound.write_all(&0u64.to_le_bytes()).await?; // cursor_id
        inbound.write_all(&0u32.to_le_bytes()).await?; // starting_from
        inbound.write_all(&1u32.to_le_bytes()).await?; // number_returned

        // documents
        inbound.write_all(&docs).await?;
        inbound.flush().await?;

        log(
            id,
            "txt",
            format!("response-oxide-{cmd}", cmd = query.command()),
            docs.as_slice(),
        )
        .await;

        debug!("OP_QUERY: [{cmd}] => {doc:?}", cmd = query.command());

        Ok(())
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
