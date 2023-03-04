use std::error::Error;
use std::sync::atomic::{AtomicI32, Ordering};

use mongodb_wire_protocol_parser::{parse, OpCode};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{instrument, trace};

use crate::cli::Cli;
use crate::log::log;
use crate::message::{OpMsg, OpQuery, OpReply};

#[derive(Debug, Clone)]
pub struct Server {
    cli: Cli,
}

#[derive(Debug)]
pub struct Connection {
    id: i32,
    cli: Cli,
    stream: TcpStream,
}

impl Server {
    pub fn new(cli: Cli) -> Self {
        Self { cli }
    }

    pub async fn start(self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(&self.cli.listen).await?;
        tracing::info!("server listening on {}", &self.cli.listen);

        if let Some(proxies) = &self.cli.proxy {
            tracing::info!("proxying to: {proxies:?}");
        }

        let id = AtomicI32::new(0);
        while let Ok((inbound, _)) = listener.accept().await {
            tracing::trace!("accepted connection from: {}", inbound.peer_addr()?);
            let new_id = id.fetch_add(1, Ordering::SeqCst) + 1;

            let handler = {
                let request = Connection::new(new_id, self.cli.clone(), inbound);
                async move {
                    let result = request.handle().await;
                    if let Err(e) = result {
                        tracing::error!("error: {e}");
                    }
                }
            };

            tokio::spawn(handler);
        }

        Ok(())
    }
}

impl Connection {
    pub fn new(id: i32, cli: Cli, stream: TcpStream) -> Self {
        Self { id, cli, stream }
    }

    #[instrument(
        name = "handle",
        skip(self)
        fields(id = %self.id),
    )]
    async fn handle(mut self) -> Result<(), Box<dyn Error>> {
        // TODO create a request struct for each request on the loop
        let main_id = self.id;
        let mut local_id = 0;
        loop {
            local_id += 1;

            let id = format!("{main_id}-{local_id}");
            let mut data = Vec::new();

            let mut buf = [0; 4];
            let n = self.stream.peek(&mut buf).await?;
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
                let n = self.stream.read(&mut buf).await?;
                if n == 0 {
                    break;
                }
                data.extend_from_slice(&buf[..n]);
                if n < buf.len() {
                    break;
                }
            }

            // send data to proxies
            self.send_to_proxies(&id, &data).await?;

            trace!("DATA = {data:?}");
            let msg = parse(data)?;
            trace!("MSG = {msg:?}");

            log(&id, "txt", "request", format!("{msg:#?}").as_bytes()).await;

            let cmd = msg.command();
            let response = match msg {
                OpCode::OpMsg(msg) => OpMsg(msg).handle().await?,
                OpCode::OpQuery(query) => OpQuery(query).handle().await?,
            };
            self.stream.write_all(&response).await?;

            log(
                id,
                "bin",
                format!("response-oxide-{cmd}"),
                response.as_slice(),
            )
            .await;
            // TODO log the json document
            // let json = serde_json::to_string_pretty(&doc)?;
            // log(id, "json", format!("response-oxide-{cmd}"), json.as_bytes()).await;
        }

        Ok(())
    }

    async fn send_to_proxies(&self, id: &str, data: &[u8]) -> Result<(), Box<dyn Error>> {
        let Some(proxies) = &self.cli.proxy else {
            return Ok(());
        };

        for (i, proxy) in proxies.iter().enumerate() {
            let mut proxy = TcpStream::connect(proxy).await?;
            proxy.write_all(data).await?;
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

            log(id, "bin", "request", data).await;
            log(id, "bin", format!("response-{i}"), &proxy_data).await;
            log(
                id,
                "txt",
                format!("response-{i}"),
                format!("{msg:#?}").as_bytes(),
            )
            .await;
            log(id, "json", format!("response-{i}"), json.as_bytes()).await;
        }

        Ok(())
    }
}
