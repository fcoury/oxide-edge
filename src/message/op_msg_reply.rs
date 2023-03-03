use std::error::Error;

use bson::{ser, Document};
use mongodb_wire_protocol_parser::{MsgHeader, Section};

pub struct OpMsgReply {
    header: MsgHeader,
    flags: u32,
    sections: Vec<Section>,
    #[allow(dead_code)]
    checksum: Option<u32>,
}

impl OpMsgReply {
    pub fn add_document(&mut self, doc: &Document) -> Result<(), Box<dyn Error>> {
        let docs = ser::to_vec(&doc)?;
        let bson_data: &[u8] = &docs;
        let section = Section::new(0, bson_data);
        self.sections.extend_from_slice(&section);

        Ok(())
    }
}

impl Into<Vec<u8>> for OpMsgReply {
    fn into(self) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        buffer.extend_from_slice(&self.header.request_id.to_le_bytes());
        buffer.extend_from_slice(&self.header.response_to.to_le_bytes());
        buffer.extend_from_slice(&self.header.op_code.to_le_bytes());
        buffer.extend_from_slice(&self.flags.to_le_bytes());
        buffer.extend_from_slice(&self.sections.get(0).unwrap().kind().to_le_bytes());
        let section = self.sections.get(0).unwrap();
        buffer.extend_from_slice(&section.kind().to_le_bytes());

        let docs = section.documents();
        let doc = docs.get(0).unwrap();
        let bson_data = ser::to_vec(&doc).unwrap();
        buffer.extend_from_slice(&bson_data);

        // TODO: checksum

        let message_length = buffer.len() as u32 + 4 as u32;
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
            op_code: 1,
        };

        OpMsgReply {
            header,
            flags: msg.flags,
            sections: Vec::new(),
            checksum: msg.checksum,
        }
    }
}

//     pub fn to_bytes(&mut self) -> Vec<u8> {
//         let mut buffer: Vec<u8> = Vec::new();
//         buffer.extend_from_slice(&0u32.to_le_bytes()); // request_id
//         buffer.extend_from_slice(&mut self.header.request_id.to_le_bytes()); // response_to
//         buffer.extend_from_slice(&mut 1u32.to_le_bytes()); // opcode - OP_REPLY = 1
//         buffer.extend_from_slice(&mut self.flags.to_le_bytes()); // flags
//         buffer.extend_from_slice(&mut self.sections.get(0).unwrap().kind().to_le_bytes()); // section kind
//         let section = self.sections.get(0).unwrap();
//         buffer.extend_from_slice(&mut section.kind().to_le_bytes());
//         let bson_data: &[u8] = &docs;
//         buffer.extend_from_slice(bson_data);

//         let message_length = buffer.len() as u32 + 4 as u32;
//         buffer.splice(..0, message_length.to_le_bytes().iter().cloned());

//         buffer
//     }
// }

// impl Into<OpMsgReply> for mongodb_wire_protocol_parser::OpMsg {
//     fn into(self) -> OpMsgReply {
//         let mut header = MsgHeader {
//             message_length: 0,
//             request_id: 0,
//             response_to: self.header.request_id,
//             op_code: 1,
//         };

//         let mut sections = Vec::new();

//         OpMsgReply {
//             header,
//             flags: self.flags,
//             sections: sections,
//             checksum: self.checksum,
//         }
//     }
// }
