use std::error::Error;

use bson::Document;
use mongodb_wire_protocol_parser::{MsgHeader, Section};

use super::{op_msg_reply::OpMsgReply, OpQueryReply};

#[derive(Debug, Clone)]
pub enum OpReply {
    OpQuery(OpQueryReply),
    OpMsg(OpMsgReply),
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
            OpReply::OpQuery(op_query_reply) => op_query_reply.documents(),
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
