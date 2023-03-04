use std::error::Error;

use clap::Parser;
use server::Server;

use crate::cli::Cli;

mod cli;
mod command;
mod error;
mod log;
mod message;
mod query;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    let server = Server::new(cli);
    server.start().await
}
