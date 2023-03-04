use std::error::Error;

use bson::{ser, Document};
use mongodb_wire_protocol_parser::{MsgHeader, Section};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpMsgReply {
    pub header: MsgHeader,
    pub flags: u32,
    pub sections: Vec<Section>,
    pub checksum: Option<u32>,
}

impl OpMsgReply {
    pub fn add_document(&mut self, doc: &Document) -> Result<(), Box<dyn Error>> {
        let docs = ser::to_vec(&doc)?;
        let bson_data: &[u8] = &docs;
        let section = Section::new(0, bson_data);
        self.sections.extend_from_slice(&section);

        Ok(())
    }

    pub fn documents(&self) -> Vec<Document> {
        let mut docs: Vec<Document> = Vec::new();
        for section in &self.sections {
            docs.extend(section.documents());
        }

        docs
    }
}

impl From<OpMsgReply> for Vec<u8> {
    fn from(reply: OpMsgReply) -> Self {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&reply.header.request_id.to_le_bytes());
        buffer.extend_from_slice(&reply.header.response_to.to_le_bytes());
        buffer.extend_from_slice(&reply.header.op_code.to_le_bytes());
        buffer.extend_from_slice(&reply.flags.to_le_bytes());
        let section = reply.sections.get(0).unwrap();
        buffer.extend_from_slice(&section.kind().to_le_bytes());

        let docs = section.documents();
        let doc = docs.get(0).unwrap();
        let bson_data = ser::to_vec(&doc).unwrap();
        buffer.extend_from_slice(&bson_data);

        // TODO: checksum

        let message_length = buffer.len() as u32 + 4_u32;
        buffer.splice(..0, message_length.to_le_bytes().iter().cloned());

        buffer
    }
}

impl From<mongodb_wire_protocol_parser::OpMsg> for OpMsgReply {
    fn from(msg: mongodb_wire_protocol_parser::OpMsg) -> Self {
        let header = MsgHeader {
            message_length: 0,
            request_id: 0,
            response_to: msg.header.request_id,
            op_code: msg.header.op_code,
        };

        OpMsgReply {
            header,
            flags: msg.flags,
            sections: Vec::new(),
            checksum: msg.checksum,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::message::OpReply;

    #[test]
    fn test_op_msg_reply_parsing() {
        let data = fs::read("tests/fixtures/op_msg_reply.bin").unwrap();
        let reply = OpReply::parse(&data).unwrap();
        let OpReply::OpMsg(reply) = reply else {
            panic!("Expected OpMsgReply");
        };
        assert_eq!(reply.header.request_id, 1447);
        assert_eq!(reply.header.response_to, 6);
        assert_eq!(reply.header.op_code, 2013);
        assert_eq!(reply.flags, 0);
        assert_eq!(reply.sections.len(), 1);
        let docs = reply.documents();
        let doc = docs.get(0).unwrap();
        assert_eq!(doc.get_str("version").unwrap(), "6.0.4");
    }
}
