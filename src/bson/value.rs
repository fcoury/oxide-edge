use super::{Array, Document};

#[derive(Debug)]
pub enum Value {
    Double(f64),                               // \x01
    String(String),                            // \x02
    Document(Document),                        // \x03
    Array(Array),                              // \x04
    Binary(Vec<u8>),                           // \x05
    Undefined,                                 // \x06
    ObjectId(Vec<u8>),                         // \x07
    Boolean(bool),                             // \x08
    UtcDateTime(u64),                          // \x09
    Null,                                      // \x0A
    Regex(String, String),                     // \x0B
    DBPointer(String, Vec<u8>),                // \x0C
    JavaScriptCode(String),                    // \x0D
    Symbol(String),                            // \x0E
    JavaScriptCodeWithScope(String, Document), // \x0F
    Int32(i32),                                // \x10
    Timestamp(u64),                            // \x11
    Int64(i64),                                // \x12
    Decimal128(Decimal128),                    // \x13
    MinKey,                                    // \xFF
    MaxKey,                                    // \x7F
}

impl Value {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Value::Double(v) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            Value::String(v) => {
                bytes.push(0x02);
                bytes.extend_from_slice(&(v.len() as i32).to_le_bytes());
                bytes.extend_from_slice(v.as_bytes());
                bytes.push(0);
            }
            Value::Document(v) => {
                bytes.push(0x03);
                bytes.extend_from_slice(&v.to_bytes());
            }
            Value::Array(v) => {
                bytes.push(0x04);
                bytes.extend_from_slice(&v.to_bytes());
            }
            Value::Binary(v) => {
                bytes.push(0x05);
                bytes.extend_from_slice(&(v.len() as i32).to_le_bytes());
                bytes.push(0);
                bytes.extend_from_slice(v);
            }
            Value::Undefined => {
                bytes.push(0x06);
            }
            Value::ObjectId(v) => {
                bytes.push(0x07);
                bytes.extend_from_slice(v);
            }
            Value::Boolean(v) => {
                bytes.push(0x08);
                bytes.push(if *v { 0x01 } else { 0x00 });
            }
            Value::UtcDateTime(v) => {
                bytes.push(0x09);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            Value::Null => {
                bytes.push(0x0A);
            }
            Value::Regex(v1, v2) => {
                bytes.push(0x0B);
                bytes.extend_from_slice(v1.as_bytes());
                bytes.push(0);
                bytes.extend_from_slice(v2.as_bytes());
                bytes.push(0);
            }
            Value::DBPointer(v1, v2) => {
                bytes.push(0x0C);
                bytes.extend_from_slice(&(v1.len() as i32).to_le_bytes());
                bytes.extend_from_slice(v1.as_bytes());
                bytes.push(0);
                bytes.extend_from_slice(v2);
            }
            Value::JavaScriptCode(v) => {
                bytes.push(0x0D);
                bytes.extend_from_slice(&(v.len() as i32).to_le_bytes());
                bytes.extend_from_slice(v.as_bytes());
                bytes.push(0);
            }
            Value::Symbol(v) => {
                bytes.push(0x0E);
                bytes.extend_from_slice(&(v.len() as i32).to_le_bytes());
                bytes.extend_from_slice(v.as_bytes());
                bytes.push(0);
            }
            Value::JavaScriptCodeWithScope(v1, v2) => {
                bytes.push(0x0F);
                bytes.extend_from_slice(&(v1.len() as i32).to_le_bytes());
                bytes.extend_from_slice(v1.as_bytes());
                bytes.push(0);
                bytes.extend_from_slice(&v2.to_bytes());
            }
            Value::Int32(v) => {
                bytes.push(0x10);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            Value::Timestamp(v) => {
                bytes.push(0x11);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            Value::Int64(v) => {
                bytes.push(0x12);
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            Value::Decimal128(_) => {
                bytes.push(0x13);
            }
            Value::MinKey => {
                bytes.push(0xFF);
            }
            Value::MaxKey => {
                bytes.push(0x7F);
            }
        }
        bytes
    }
}

#[derive(Debug)]
pub struct Decimal128;
