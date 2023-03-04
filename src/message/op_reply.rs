use std::error::Error;

use bson::{ser, Document};
use mongodb_wire_protocol_parser::{MsgHeader, Section};

#[derive(Debug, Clone)]
pub enum OpReply {
    OpQuery(OpQueryReply),
    OpMsg(OpMsgReply),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpQueryReply {
    header: MsgHeader,
    flags: u32,
    cursor_id: u64,
    starting_from: u32,
    number_returned: u32,
    documents: Vec<Document>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct OpMsgReply {
    header: MsgHeader,
    flags: u32,
    sections: Vec<Section>,
    checksum: Option<u32>,
}

impl OpReply {
    pub fn parse(input: &[u8]) -> Result<OpReply, Box<dyn Error>> {
        let mut cursor = 0;
        let header = MsgHeader::from_slice(&input[cursor..])?;
        cursor += 16;

        match header.op_code {
            1 => {
                return Ok(OpReply::OpQuery(Self::parse_op_query_reply(
                    header,
                    &input[cursor..],
                )?));
            }
            2013 => {
                return Ok(OpReply::OpMsg(Self::parse_op_msg_reply(
                    header,
                    &input[cursor..],
                )?));
            }
            _ => {
                unimplemented!("parsing for op_code {} not implemented", header.op_code);
            }
        }
    }

    pub fn documents(&self) -> Vec<Document> {
        match self {
            OpReply::OpQuery(op_query_reply) => op_query_reply.documents.clone(),
            OpReply::OpMsg(op_msg_reply) => op_msg_reply.documents(),
        }
    }

    pub fn parse_op_query_reply(
        header: MsgHeader,
        input: &[u8],
    ) -> Result<OpQueryReply, Box<dyn Error>> {
        let mut cursor = 0;

        let flags = u32::from_le_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
        ]);
        cursor += 4;

        let cursor_id = u64::from_le_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
            input[cursor + 4],
            input[cursor + 5],
            input[cursor + 6],
            input[cursor + 7],
        ]);
        cursor += 8;

        let starting_from = u32::from_le_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
        ]);
        cursor += 4;

        let number_returned = u32::from_le_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
        ]);
        cursor += 4;

        let mut documents: Vec<Document> = Vec::new();
        for _ in 0..number_returned {
            let doc: Document = bson::from_slice(&input[cursor..])?;
            cursor += doc.len();
            documents.push(doc);
        }

        Ok(OpQueryReply {
            header,
            flags,
            cursor_id,
            starting_from,
            number_returned,
            documents,
        })
    }

    pub fn parse_op_msg_reply(
        header: MsgHeader,
        input: &[u8],
    ) -> Result<OpMsgReply, Box<dyn Error>> {
        let mut cursor = 0;
        let flags = u32::from_le_bytes([
            input[cursor],
            input[cursor + 1],
            input[cursor + 2],
            input[cursor + 3],
        ]);
        cursor += 4;

        let (section, checksum) = Section::from_slice(&input[cursor..])?;
        let sections = vec![section];

        Ok(OpMsgReply {
            header,
            flags,
            sections,
            checksum,
        })
    }
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

    use super::*;

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
        println!("{:?}", doc);
        assert_eq!(doc.get_bool("helloOk").unwrap(), true);
        assert_eq!(doc.get_bool("ismaster").unwrap(), true);
    }
}
