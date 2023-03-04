use std::error::Error;

use mongodb_wire_protocol_parser::{parse, OpCode};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::{debug, instrument, trace};

use crate::{
    cli::Cli,
    log::log,
    message::{OpMsg, OpQuery, OpReply},
};

#[derive(Debug)]
pub struct Connection {
    id: i32,
    cli: Cli,
    stream: TcpStream,
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
    pub async fn handle(mut self) -> Result<(), Box<dyn Error>> {
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
            let proxy_responses = self.send_to_proxies(&id, &data).await?;

            trace!("DATA = {data:?}");
            let msg = parse(data)?;
            trace!("MSG = {msg:?}");

            self.log(&id, "txt", "request", format!("{msg:#?}").as_bytes())
                .await;

            let cmd = msg.command();
            let (reply, response) = match msg {
                OpCode::OpMsg(msg) => OpMsg(msg).handle().await?,
                OpCode::OpQuery(query) => OpQuery(query).handle().await?,
            };
            self.stream.write_all(&response).await?;

            let proxy_responses = proxy_responses
                .iter()
                .map(|data| OpReply::parse(data))
                .collect::<Result<Vec<_>, _>>()?;
            if proxy_responses.is_empty() {
                debug!("cmd={cmd} reply={reply:?}");
            } else {
                debug!("\n  cmd={cmd}\n  reply={reply:?}\n  proxies={proxy_responses:?}");
            }

            self.log(
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

    async fn send_to_proxies(&self, id: &str, data: &[u8]) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        let mut responses = vec![];
        let Some(proxies) = &self.cli.proxy else {
            return Ok(responses);
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

            self.log(id, "bin", "request", data).await;
            self.log(id, "bin", format!("response-{i}"), &proxy_data)
                .await;
            self.log(
                id,
                "txt",
                format!("response-{i}"),
                format!("{msg:#?}").as_bytes(),
            )
            .await;
            self.log(id, "json", format!("response-{i}"), json.as_bytes())
                .await;

            responses.push(proxy_data);
        }

        Ok(responses)
    }

    pub async fn log(
        &self,
        id: impl Into<String>,
        kind: impl Into<String>,
        name: impl Into<String>,
        data: &[u8],
    ) {
        if let Some(path) = &self.cli.dump {
            let id = id.into();
            let kind = kind.into();
            let name = name.into();
            log(format!("{path}/{id}-{name}.{kind}"), data).await;
        }
    }
}
