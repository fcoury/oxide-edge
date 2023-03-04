use std::{net::SocketAddr, path::PathBuf};

use clap::{Parser, Subcommand};
use mongodb_wire_protocol_parser::parse;

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
    pub trace: bool,

    /// Debugging mode logs
    #[clap(short, long)]
    pub debug: bool,

    #[clap(subcommand)]
    pub subcommand: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Parses a dump file
    Parse {
        /// The file to parse
        file: PathBuf,
    },
}

impl Cli {
    pub fn log_level(&self) -> &str {
        if self.trace {
            "trace"
        } else if self.debug {
            "debug"
        } else {
            "info"
        }
    }

    pub fn run_subcommand(&self) -> bool {
        let Some(cmd) = &self.subcommand else {
            return false;
        };

        match cmd {
            Command::Parse { file } => {
                let contents = std::fs::read(file).unwrap();
                // let header = MsgHeader::from_slice(&contents).unwrap();
                let msg = parse(contents).unwrap();
                println!("{msg:#?}");
            }
        };

        true
    }
}
