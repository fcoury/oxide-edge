use std::env;
use std::error::Error;

use futures::FutureExt;
use mongodb_wire_protocol_parser::parse;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

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

async fn handle(mut inbound: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();

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

    let msg = parse(data)?;
    tracing::info!("GOT {:?}", msg);

    Ok(())
}
