use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    #[clap(short, long, default_value = "127.0.0.1:27017")]
    pub listen: SocketAddr,

    #[clap(short, long)]
    pub proxy: Option<Vec<SocketAddr>>,
}
