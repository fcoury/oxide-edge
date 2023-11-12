use crate::bson::Document;

pub struct OpReply {
    pub response_flags: u32,
    pub cursor_id: i64,
    pub starting_from: i32,
    pub number_returned: i32,
    pub documents: Vec<Document>,
}

impl OpReply {
    pub fn set_flag(&mut self, flag: ResponseFlag) {
        self.response_flags |= flag as u32;
    }

    pub fn has_flag(&self, flag: &ResponseFlag) -> bool {
        self.response_flags & *flag as u32 == *flag as u32
    }

    fn to_bytes(&self, request_id: i32, response_to: i32) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Message length placeholder
        bytes.extend_from_slice(&[0u8; 4]);
        // Request id
        bytes.extend_from_slice(&request_id.to_le_bytes());
        // Response to
        bytes.extend_from_slice(&response_to.to_le_bytes());
        // Op code
        bytes.extend_from_slice(&1i32.to_le_bytes());

        // OpReply body
        bytes.extend_from_slice(&self.response_flags.to_le_bytes());
        bytes.extend_from_slice(&self.cursor_id.to_le_bytes());
        bytes.extend_from_slice(&self.starting_from.to_le_bytes());
        bytes.extend_from_slice(&(self.documents.len() as u32).to_le_bytes());

        for document in &self.documents {
            bytes.extend_from_slice(&document.to_bytes());
        }

        // Replace message length placeholder
        let length = bytes.len() as i32;
        bytes[0..4].copy_from_slice(&length.to_le_bytes());

        bytes
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum ResponseFlag {
    CursorNotFound = 1 << 0,   // 0b0001
    QueryFailure = 1 << 1,     // 0b0010
    ShardConfigStale = 1 << 2, // 0b0100
    AwaitCapable = 1 << 3,     // 0b1000
}
