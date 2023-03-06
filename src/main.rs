use std::error::Error;

use clap::Parser;
use dotenvy::dotenv;
use server::Server;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::cli::Cli;

mod cli;
mod command;
mod dal;
mod error;
mod log;
mod message;
mod query;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let cli = Cli::parse();
    let log_level = cli.log_level();
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(log_level))?,
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    if cli.run_subcommand() {
        return Ok(());
    }

    let server = Server::new(cli);
    server.start().await
}
