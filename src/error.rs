use std::sync::mpsc;

use crate::Message;

#[derive(Debug)]
pub enum Error {
    AcceptingConnection(std::io::Error),
    ReceiverHungup(mpsc::RecvError),
    SenderError(mpsc::SendError<Message>),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::AcceptingConnection(err)
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(err: mpsc::RecvError) -> Self {
        Error::ReceiverHungup(err)
    }
}

impl From<mpsc::SendError<Message>> for Error {
    fn from(err: mpsc::SendError<Message>) -> Self {
        Error::SenderError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
