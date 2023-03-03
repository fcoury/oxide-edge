use std::env;
use std::error::Error;

use bson::ser;
use futures::FutureExt;
use mongodb_wire_protocol_parser::{parse, OpCode};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, error, instrument, trace};

use crate::command::{run_op_msg, run_op_query, HEADER_SIZE};

mod command;
mod query;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let listen_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:27017".to_string());
    tracing::info!("server listening on {}", listen_addr);
    let listener = TcpListener::bind(&listen_addr).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        tracing::debug!("accepted connection from: {}", inbound.peer_addr()?);
        let handler = handle(inbound).map(|r| {
            if let Err(e) = r {
                tracing::error!("error: {}", e);
            }
        });

        tokio::spawn(handler);
    }

    Ok(())
}

#[instrument(
        name = "handle",
        skip(inbound),
        fields(
            // `%` serializes the peer IP addr with `Display`
            port = %inbound.peer_addr().unwrap().port(),
        ),
    )]
async fn handle(mut inbound: TcpStream) -> Result<(), Box<dyn Error>> {
    loop {
        let mut data = Vec::new();

        let mut buf = [0; 4];
        let n = inbound.peek(&mut buf).await?;
        if n == 0 {
            break;
        }
        let size: u32 = u32::from_le_bytes(buf);
        trace!("SIZE = {}", size);
        if size == 0 {
            break;
        }

        loop {
            let mut buf = [0; 4096];
            let n = inbound.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            data.extend_from_slice(&buf[..n]);
            if n < buf.len() {
                break;
            }
        }

        trace!("DATA = {:?}", data);
        let msg = parse(data)?;
        trace!("MSG = {:?}", msg);
        match msg {
            OpCode::OpMsg(msg) => {
                if msg.sections.len() < 1 {
                    error!("OpMsg must have at least one section");
                    return Err(Box::new(OpMsgHandlingError::InvalidOpMsg(
                        "OpMsg must have at least one section".to_string(),
                    )));
                }

                let doc = run_op_msg(msg.clone())?;

                let docs = ser::to_vec(&doc)?;
                let message_length = HEADER_SIZE + 5 + docs.len() as u32;

                // header
                inbound.write_all(&message_length.to_le_bytes()).await?;
                inbound.write_all(&0u32.to_le_bytes()).await?; // request_id
                inbound
                    .write_all(&msg.header.request_id.to_le_bytes())
                    .await?; // response_to
                inbound.write_all(&1u32.to_le_bytes()).await?; // opcode - OP_REPLY = 1

                // body
                inbound.write_all(&msg.flags.to_le_bytes()).await?; // flags

                // documents
                let section = msg.sections.get(0).unwrap();
                inbound.write_u8(section.kind()).await?;

                let bson_data: &[u8] = &docs;
                inbound.write_all(bson_data).await?;

                // TODO: checksum

                inbound.flush().await?;
                debug!("OP_MSG: [{cmd}] => {doc:?}", cmd = msg.command());
            }
            OpCode::OpQuery(query) => {
                let doc = run_op_query(query.clone())?;

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

                debug!("OP_QUERY: [{cmd}] => {doc:?}", cmd = query.command());
            }
        };
    }

    Ok(())
}

#[derive(Debug, Error)]
enum OpMsgHandlingError {
    #[error("invalid OP_MSG: {0}")]
    InvalidOpMsg(String),
}
