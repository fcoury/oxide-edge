use std::error::Error;

use clap::Parser;
use futures::FutureExt;
use mongodb_wire_protocol_parser::{parse, OpCode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{instrument, trace};

use crate::cli::Cli;
use crate::log::log;
use crate::message::{OpMsg, OpQuery, OpReply};

mod cli;
mod command;
mod error;
mod log;
mod message;
mod query;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let listen_addr = &cli.listen;

    tracing::info!("server listening on {}", listen_addr);
    let listener = TcpListener::bind(listen_addr).await?;

    let mut id = 0;
    while let Ok((inbound, _)) = listener.accept().await {
        tracing::debug!("accepted connection from: {}", inbound.peer_addr()?);
        id += 1;
        let handler = handle(id, inbound, cli.clone()).map(|r| {
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
    skip(main_id, inbound, cli),
    fields(id = %main_id),
)]
async fn handle(main_id: i32, mut inbound: TcpStream, cli: Cli) -> Result<(), Box<dyn Error>> {
    let mut local_id = 0;
    loop {
        local_id += 1;

        let id = format!("{}-{}", main_id, local_id);
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

                let mut proxy_data = vec![0; size as usize];
                proxy.read_exact(&mut proxy_data).await?;
                trace!("proxy {i} data = {:?}", proxy_data);

                let msg = OpReply::parse(&proxy_data)?;
                let json = serde_json::to_string_pretty(&msg.documents())?;

                log(&id, "bin", "request", &data).await;
                log(&id, "bin", format!("response-{i}"), &proxy_data).await;
                log(
                    &id,
                    "txt",
                    format!("response-{i}"),
                    format!("{:#?}", msg).as_bytes(),
                )
                .await;
                log(&id, "json", format!("response-{i}"), json.as_bytes()).await;
            }
        }

        trace!("DATA = {:?}", data);
        let msg = parse(data)?;
        trace!("MSG = {:?}", msg);

        log(&id, "txt", "request", format!("{:#?}", msg).as_bytes()).await;

        match msg {
            OpCode::OpMsg(msg) => OpMsg(msg).handle(&id, &mut inbound).await?,
            OpCode::OpQuery(query) => OpQuery(query).handle(&id, &mut inbound).await?,
        };
    }

    Ok(())
}
