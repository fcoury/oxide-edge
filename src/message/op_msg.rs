use anyhow::anyhow;
use bson::Document;
use mongodb_wire_protocol_parser::OpCode;
use tracing::{debug, error, instrument, trace};

use crate::{
    command::{buildinfo, error, getcmdlineopts, getparameter, hello, ismaster, ping},
    error::Error,
    message::OpMsgReply,
};

#[derive(Debug)]
pub struct OpMsg(pub mongodb_wire_protocol_parser::OpMsg);

impl OpMsg {
    #[instrument(name = "OpMsg.handle", skip(self))]
    pub async fn handle(self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let cmd = &self.0.command();
        let doc = &self.run().await?;
        let reply = self.reply(doc)?;
        debug!("OpMsg cmd={cmd} reply={reply:?}");

        let data: Vec<u8> = reply.into();

        Ok(data)
    }

    pub fn reply(self, doc: &Document) -> Result<OpMsgReply, Box<dyn std::error::Error>> {
        let mut reply: OpMsgReply = self.0.into();
        reply.add_document(doc)?;

        Ok(reply)
    }

    async fn run(&self) -> anyhow::Result<Document> {
        if self.0.sections.is_empty() {
            error!("OpMsg must have at least one section");
            return Err(anyhow!(Error::InvalidOpMsg(
                "OpMsg must have at least one section".to_string(),
            )));
        }

        let command = self.0.command();
        trace!("OP_MSG command: {}", command.to_ascii_lowercase());

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
                error::run(Box::new(Error::UnknownCommand(command.to_string())))?
            }
        };

        Ok(doc)
    }
}
