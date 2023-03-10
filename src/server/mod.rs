use std::error::Error;
use std::sync::atomic::{AtomicI32, Ordering};

use duckdb::DuckdbConnectionManager;
use tokio::fs;
use tokio::net::TcpListener;

use crate::cli::Cli;
use crate::server::connection::Connection;

mod connection;

#[derive(Debug, Clone)]
pub struct Server {
    cli: Cli,
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

        if let Some(path) = &self.cli.db.parent() {
            fs::create_dir_all(path).await?;
        }
        let manager = DuckdbConnectionManager::file(&self.cli.db)?;
        let pool = r2d2::Pool::new(manager)?;
        pool.get()?.execute_batch("INSTALL 'json'; LOAD 'json';")?;

        let id = AtomicI32::new(0);
        while let Ok((inbound, _)) = listener.accept().await {
            tracing::trace!("accepted connection from: {}", inbound.peer_addr()?);
            let new_id = id.fetch_add(1, Ordering::SeqCst) + 1;

            let handler = {
                let connection = Connection::new(new_id, self.cli.clone(), &pool, inbound);
                async move {
                    let result = connection.handle().await;
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
