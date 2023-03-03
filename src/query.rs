use mongodb_wire_protocol_parser::OpQuery;
use tracing::info;

pub trait Handler {
    fn handle(&self);
}

impl Handler for OpQuery {
    fn handle(&self) {
        let command = match self.query.keys().next() {
            Some(key) => key,
            None => "ismaster",
        };
        info!("OP_QUERY command: {}", command);
    }
}
