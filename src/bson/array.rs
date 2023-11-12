use super::{Document, Value};

#[derive(Debug)]
pub struct Array(pub Vec<Value>);

impl Array {
    pub fn from_document(d: Document) -> Self {
        let mut array = Vec::new();
        for (_, value) in d.0 {
            array.push(value);
        }
        Self(array)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for value in &self.0 {
            bytes.extend_from_slice(&value.to_bytes());
        }
        bytes.push(0);
        bytes
    }
}
