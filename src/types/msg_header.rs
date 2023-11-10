use std::fmt;

pub struct MsgHeader<'a> {
    bytes: &'a [u8],
}

impl<'a> MsgHeader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn message_length(&self) -> i32 {
        i32::from_le_bytes(
            self.bytes[0..4]
                .try_into()
                .expect("message must have at least 4 bytes"),
        )
    }

    pub fn request_id(&self) -> i32 {
        i32::from_le_bytes(self.bytes[4..8].try_into().expect("message is well formed"))
    }

    pub fn response_to(&self) -> i32 {
        i32::from_le_bytes(
            self.bytes[8..12]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn op_code(&self) -> i32 {
        i32::from_le_bytes(
            self.bytes[12..16]
                .try_into()
                .expect("message is well formed"),
        )
    }
}

impl fmt::Debug for MsgHeader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MsgHeader")
            .field("message_length", &self.message_length())
            .field("request_id", &self.request_id())
            .field("response_to", &self.response_to())
            .field("op_code", &self.op_code())
            .finish()
    }
}
