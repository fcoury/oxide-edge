use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{mpsc, Arc},
    thread,
};

mod error;
mod types;

use crate::{
    error::Result,
    types::{MsgHeader, OpQuery},
};

#[derive(Debug)]
pub enum Message {
    ClientConnected {
        stream: Arc<TcpStream>,
        addr: SocketAddr,
    },
    ClientDisconnected {
        addr: SocketAddr,
    },
    NewMessage {
        addr: SocketAddr,
        bytes: Box<[u8]>,
    },
}

struct Client {
    stream: Arc<TcpStream>,
}

struct Server {
    clients: HashMap<SocketAddr, Client>,
}

impl Server {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    fn client_connected(&mut self, stream: Arc<TcpStream>, addr: SocketAddr) {
        let client = Client {
            stream: stream.clone(),
        };
        self.clients.insert(addr, client);
        _ = write!(stream.as_ref(), "Hello");
    }

    fn client_disconnected(&mut self, addr: SocketAddr) {
        println!("Client disconnected: {}", addr);
        self.clients.remove(&addr);
    }

    fn new_message(&mut self, addr: SocketAddr, bytes: &[u8]) {
        println!("New message from {}: {:?}", addr, bytes);
        let header = MsgHeader::new(bytes);
        println!("message: {:#?}", header);

        match header.op_code() {
            2004 => {
                let op_query = OpQuery::new(bytes);
                println!("op_query: {:#?}", op_query);
            }
            op_code => {
                unimplemented!("op_code: {}", op_code);
            }
        }
    }
}

fn client(stream: Arc<TcpStream>, tx: mpsc::Sender<Message>) -> Result<()> {
    let addr = stream.peer_addr()?;

    tx.send(Message::ClientConnected {
        stream: stream.clone(),
        addr,
    })?;

    let mut buffer = [0; 1024];
    loop {
        let n = stream.as_ref().read(&mut buffer)?;
        if n > 0 {
            let bytes = buffer[..n].iter().cloned().collect();
            println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
            tx.send(Message::NewMessage { addr, bytes })?;
        } else {
            tx.send(Message::ClientDisconnected { addr })?;
            break;
        }
    }

    Ok(())
}

fn server(rx: mpsc::Receiver<Message>) -> Result<()> {
    let mut server = Server::new();
    loop {
        let msg = rx.recv()?;
        println!("Message: {:?}", msg);

        match msg {
            Message::ClientConnected { stream, addr } => server.client_connected(stream, addr),
            Message::ClientDisconnected { addr } => server.client_disconnected(addr),
            Message::NewMessage { addr, bytes } => server.new_message(addr, &bytes),
        }
    }
}

static DEFAULT_ADDR: &str = "127.0.0.1:27018";

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let addr = match args.get(1) {
        Some(addr) => addr,
        None => DEFAULT_ADDR,
    };
    let listener = TcpListener::bind(addr)?;
    println!("Listening on: {}...", addr);
    let (tx, rx) = mpsc::channel();

    thread::spawn(|| server(rx));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let stream = Arc::new(stream);
                let tx = tx.clone();
                thread::spawn(|| client(stream, tx));
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }

    Ok(())
}
