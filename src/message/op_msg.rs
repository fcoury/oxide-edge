use std::error::Error;

use anyhow::anyhow;
use bson::{ser, Document};
use mongodb_wire_protocol_parser::OpCode;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{debug, error};

use crate::{
    command::{
        buildinfo, error, getcmdlineopts, getparameter, hello, ismaster, ping, CommandError,
        HEADER_SIZE,
    },
    OpMsgHandlingError,
};

pub struct OpMsg(pub mongodb_wire_protocol_parser::OpMsg);

impl OpMsg {
    pub async fn handle(self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let doc = self.run().await?;

        let msg = self.0;
        let docs = ser::to_vec(&doc)?;
        let message_length = HEADER_SIZE + 5 + docs.len() as u32;

        // header
        stream.write_all(&message_length.to_le_bytes()).await?;
        stream.write_all(&0u32.to_le_bytes()).await?; // request_id
        stream
            .write_all(&msg.header.request_id.to_le_bytes())
            .await?; // response_to
        stream.write_all(&1u32.to_le_bytes()).await?; // opcode - OP_REPLY = 1

        // body
        stream.write_all(&msg.flags.to_le_bytes()).await?; // flags

        // documents
        let section = msg.sections.get(0).unwrap();
        stream.write_u8(section.kind()).await?;

        let bson_data: &[u8] = &docs;
        stream.write_all(bson_data).await?;

        // TODO: checksum

        stream.flush().await?;
        debug!("OP_MSG: [{cmd}] => {doc:?}", cmd = msg.command());

        Ok(())
    }

    async fn run(&self) -> anyhow::Result<Document> {
        if self.0.sections.len() < 1 {
            error!("OpMsg must have at least one section");
            return Err(anyhow!(OpMsgHandlingError::InvalidOpMsg(
                "OpMsg must have at least one section".to_string(),
            )));
        }

        let command = self.0.command();
        debug!("OP_MSG command: {}", command.to_ascii_lowercase());

        let msg = OpCode::OpMsg(self.0.clone());
        let doc = match command.to_ascii_lowercase().as_ref() {
            "ismaster" => ismaster::run(msg)?,
            "buildinfo" => buildinfo::run(msg)?,
            "getcmdlineopts" => getcmdlineopts::run(msg)?,
            "getparameter" => getparameter::run(msg)?,
            "ping" => ping::run()?,
            "hello" => hello::run()?,
            _ => {
                error!("unknown command: {}", command);
                error::run(Box::new(CommandError::UnknownCommand(command.to_string())))?
            }
        };

        Ok(doc)
    }
}
