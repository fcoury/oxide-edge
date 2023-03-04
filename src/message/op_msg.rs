use anyhow::anyhow;
use bson::Document;
use mongodb_wire_protocol_parser::OpCode;
use tracing::{debug, error, instrument};

use crate::{
    command::{
        buildinfo, error, getcmdlineopts, getparameter, hello, ismaster, ping, CommandError,
    },
    error::Error,
    message::OpMsgReply,
};

#[derive(Debug)]
pub struct OpMsg(pub mongodb_wire_protocol_parser::OpMsg);

impl OpMsg {
    #[instrument(name = "OpMsg.handle", skip(self))]
    pub async fn handle(self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let doc = &self.run().await?;
        let cmd = &self.command();
        let reply = self.reply(doc)?;

        let data: Vec<u8> = reply.into();

        debug!("OP_MSG: [{cmd}] => ({size}) {doc:?}", size = data.len());

        Ok(data)
    }

    pub fn command(&self) -> String {
        self.0.command()
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
