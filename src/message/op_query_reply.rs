use bson::{ser, Document};
use mongodb_wire_protocol_parser::MsgHeader;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpQueryReply {
    pub header: MsgHeader,
    pub flags: u32,
    pub cursor_id: u64,
    pub starting_from: u32,
    pub number_returned: u32,
    pub documents: Vec<Document>,
}

impl OpQueryReply {
    pub fn add_document(&mut self, doc: &Document) {
        self.documents = vec![doc.clone()];
    }

    pub fn documents(&self) -> Vec<Document> {
        self.documents.clone()
    }
}

impl From<OpQueryReply> for Vec<u8> {
    fn from(reply: OpQueryReply) -> Self {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&reply.header.request_id.to_le_bytes());
        buffer.extend_from_slice(&reply.header.response_to.to_le_bytes());
        buffer.extend_from_slice(&reply.header.op_code.to_le_bytes());
        buffer.extend_from_slice(&reply.flags.to_le_bytes());
        buffer.extend_from_slice(&reply.cursor_id.to_le_bytes());
        buffer.extend_from_slice(&reply.starting_from.to_le_bytes());
        buffer.extend_from_slice(&reply.number_returned.to_le_bytes());

        for doc in &reply.documents {
            let bson_data = ser::to_vec(&doc).unwrap();
            buffer.extend_from_slice(&bson_data);
        }

        let message_length = buffer.len() as u32 + 4_u32;
        buffer.splice(..0, message_length.to_le_bytes().iter().cloned());

        buffer
    }
}

impl From<mongodb_wire_protocol_parser::OpQuery> for OpQueryReply {
    fn from(query: mongodb_wire_protocol_parser::OpQuery) -> Self {
        let header = MsgHeader {
            message_length: 0,
            request_id: 0,
            response_to: query.header.request_id,
            op_code: 1, // OP_REPLY
        };

        OpQueryReply {
            header,
            flags: query.flags,
            cursor_id: 0,
            starting_from: 0,
            number_returned: 0,
            documents: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::message::OpReply;

    #[test]
    fn test_op_query_reply_parsing() {
        let data = fs::read("tests/fixtures/op_query_reply.bin").unwrap();
        let reply = OpReply::parse(&data).unwrap();
        let OpReply::OpQuery(reply) = reply else {
            panic!("Expected OpQueryReply");
        };
        assert_eq!(reply.header.request_id, 1446);
        assert_eq!(reply.header.response_to, 7);
        assert_eq!(reply.header.op_code, 1);
        assert_eq!(reply.flags, 8);
        assert_eq!(reply.cursor_id, 0);
        assert_eq!(reply.starting_from, 0);
        assert_eq!(reply.number_returned, 1);
        let doc = reply.documents.get(0).unwrap();
        println!("{doc:?}");
        assert!(doc.get_bool("helloOk").unwrap());
        assert!(doc.get_bool("ismaster").unwrap());
    }
}
