use std::error::Error;

use anyhow::anyhow;
use bson::Document;
use mongodb_wire_protocol_parser::OpCode;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tracing::{debug, error};

use crate::{
    command::{
        buildinfo, error, getcmdlineopts, getparameter, hello, ismaster, ping, CommandError,
    },
    OpMsgHandlingError,
};

use super::op_msg_reply::OpMsgReply;

pub struct OpMsg(pub mongodb_wire_protocol_parser::OpMsg);

impl OpMsg {
    pub async fn handle(self, stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
        let doc = &self.run().await?;
        let cmd = &self.0.command();
        let reply = self.reply(doc)?;

        let data: Vec<u8> = reply.into();

        debug!("OP_MSG: [{cmd}] => ({size}) {doc:?}", size = data.len());

        stream.write_all(&data).await?;

        Ok(())
    }

    pub fn reply(self, doc: &Document) -> Result<OpMsgReply, Box<dyn Error>> {
        let mut reply: OpMsgReply = self.0.into();
        reply.add_document(doc)?;

        Ok(reply)
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
