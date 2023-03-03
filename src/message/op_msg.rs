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
    },
    OpMsgHandlingError,
};

pub struct OpMsg(pub mongodb_wire_protocol_parser::OpMsg);

impl OpMsg {
    pub async fn handle(self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let doc = self.run().await?;

        let msg = self.0;
        let docs = ser::to_vec(&doc)?;

        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&0u32.to_le_bytes());
        buffer.extend_from_slice(&mut msg.header.request_id.to_le_bytes());
        buffer.extend_from_slice(&mut 1u32.to_le_bytes());
        buffer.extend_from_slice(&mut msg.flags.to_le_bytes());
        buffer.extend_from_slice(&mut msg.sections.get(0).unwrap().kind().to_le_bytes());
        let section = msg.sections.get(0).unwrap();
        buffer.extend_from_slice(&mut section.kind().to_le_bytes());
        let bson_data: &[u8] = &docs;
        buffer.extend_from_slice(bson_data);

        debug!("len before: {:?}", buffer.len());
        let message_length = buffer.len() as u32 + 4 as u32;
        buffer.splice(..0, message_length.to_le_bytes().iter().cloned());
        debug!("len after: {:?}", buffer.len());

        // TODO: checksum

        stream.write_all(&buffer).await?;

        stream.flush().await?;
        debug!(
            "OP_MSG: [{cmd}] => ({size} - {message_length}) {doc:?}",
            size = buffer.len(),
            cmd = msg.command()
        );

        Ok(())

        // // header
        // stream.write_all(&message_length.to_le_bytes()).await?; // 4
        // stream.write_all(&0u32.to_le_bytes()).await?; // request_id (4)
        // stream
        //     .write_all(&msg.header.request_id.to_le_bytes())
        //     .await?; // response_to (4)
        // stream.write_all(&1u32.to_le_bytes()).await?; // opcode - OP_REPLY = 1 (4)

        // // body
        // stream.write_all(&msg.flags.to_le_bytes()).await?; // flags (4)

        // // documents
        // let section = msg.sections.get(0).unwrap();
        // stream.write_u8(section.kind()).await?;

        // let bson_data: &[u8] = &docs;
        // stream.write_all(bson_data).await?;
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
