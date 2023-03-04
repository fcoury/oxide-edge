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

    /// Defines the DuckDB database file to use
    #[clap(short, long, value_name = "DB_FILE", env = "DATABASE_FILE")]
    pub db: PathBuf,

    /// Dump requests to files on the DUMP folder
    #[clap(short = 'u', long)]
    pub dump: Option<Option<PathBuf>>,

    /// Tracing mode logs
    #[clap(long, conflicts_with = "debug")]
    pub trace: bool,

    /// Debugging mode logs
    #[clap(long)]
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

    pub fn dump_path(&self) -> Option<String> {
        let path = match &self.dump {
            Some(Some(path)) => Some(path.into()),
            Some(None) => Some(PathBuf::from("dump")),
            None => None,
        };

        if let Some(path) = path {
            if !path.exists() {
                std::fs::create_dir_all(&path).unwrap();
            }
            Some(path.to_str().unwrap().to_string())
        } else {
            None
        }
    }
}
