use super::Document;

pub struct Bson<'a> {
    bytes: &'a [u8],
}

impl<'a> Bson<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn len(&self) -> i32 {
        i32::from_le_bytes(
            self.bytes[0..4]
                .try_into()
                .expect("message must have at least 4 bytes"),
        )
    }

    pub fn parse(&self) -> Document {
        todo!()
    }
}
