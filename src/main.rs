use std::env;
use std::error::Error;

use futures::FutureExt;
use mongodb_wire_protocol_parser::{parse, OpCode};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, instrument, trace};

use crate::message::{OpMsg, OpQuery};

mod command;
mod message;
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
    fields(port = %inbound.peer_addr().unwrap().port()),
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
            OpCode::OpMsg(msg) => OpMsg(msg).handle(&mut inbound).await?,
            OpCode::OpQuery(query) => OpQuery(query).handle(&mut inbound).await?,
        };
    }

    Ok(())
}

#[derive(Debug, Error)]
enum OpMsgHandlingError {
    #[error("invalid OP_MSG: {0}")]
    InvalidOpMsg(String),
}
