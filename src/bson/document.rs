use std::collections::HashMap;

use super::Value;

#[derive(Debug)]
pub struct Document(pub HashMap<String, Value>);

impl Document {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (key, value) in &self.0 {
            bytes.extend_from_slice(&key.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&value.to_bytes());
        }
        bytes.push(0);
        bytes
    }
}
