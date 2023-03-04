use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    /// Listen on this address
    #[clap(short, long, default_value = "127.0.0.1:27017")]
    pub listen: SocketAddr,

    /// Sends requests to other backends, can be used multiple times
    #[clap(short, long)]
    pub proxy: Option<Vec<SocketAddr>>,

    /// Dump requests to files on the DUMP folder
    #[clap(short = 'u', long)]
    pub dump: Option<String>,

    /// Tracing mode logs
    #[clap(short, long, conflicts_with = "debug")]
    pub tracing: Option<bool>,

    /// Debugging mode logs
    #[clap(short, long)]
    pub debug: Option<bool>,
}

impl Cli {
    pub fn log_level(&self) -> &str {
        if self.tracing.unwrap_or(false) {
            "trace"
        } else if self.debug.unwrap_or(false) {
            "debug"
        } else {
            "info"
        }
    }
}
