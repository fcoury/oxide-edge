use std::error::Error;

use clap::Parser;
use futures::FutureExt;
use mongodb_wire_protocol_parser::{parse, OpCode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, instrument, trace};

use crate::cli::Cli;
use crate::message::{OpMsg, OpQuery, OpReply};

mod cli;
mod command;
mod error;
mod message;
mod query;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let listen_addr = &cli.listen;

    tracing::info!("server listening on {}", listen_addr);
    let listener = TcpListener::bind(listen_addr).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        tracing::debug!("accepted connection from: {}", inbound.peer_addr()?);
        let handler = handle(inbound, cli.clone()).map(|r| {
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
    skip(inbound, cli),
    fields(port = %inbound.peer_addr().unwrap().port()),
)]
async fn handle(mut inbound: TcpStream, cli: Cli) -> Result<(), Box<dyn Error>> {
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

        // send data to proxies
        if let Some(proxies) = &cli.proxy {
            for (i, proxy) in proxies.iter().enumerate() {
                let mut proxy = TcpStream::connect(proxy).await?;
                proxy.write_all(&data).await?;
                proxy.flush().await?;

                let mut buf = [0; 4];
                proxy.peek(&mut buf).await?;
                let size: u32 = u32::from_le_bytes(buf);
                trace!("proxy {i} size = {size}");

                let mut data = vec![0; size as usize];
                proxy.read_exact(&mut data).await?;
                trace!("proxy {i} data = {:?}", data);
                let response_to = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);

                log("bin", i, response_to, &data).await;

                let msg = OpReply::parse(&data)?;
                let json = serde_json::to_string_pretty(&msg.documents())?;
                log("json", i, response_to, json.as_bytes()).await;
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

async fn log(kind: &str, n: usize, id: u32, data: &[u8]) {
    let file = tokio::fs::File::create(format!("dump/proxy_{n}_{id}.{kind}")).await;
    if let Err(e) = file {
        error!("failed to create file: {}", e);
        return;
    }
    let mut file = file.unwrap();
    let result = file.write_all(data).await;
    if let Err(e) = result {
        error!("failed to write to file: {}", e);
        return;
    }
}
