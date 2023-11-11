use std::collections::HashMap;

use crate::types::document::Array;

use super::{Document, Value};

pub struct Bson<'a> {
    bytes: &'a [u8],
}

impl<'a> Bson<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

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
        self.parse_document(4)
    }

    pub fn parse_document(&self, start_from: usize) -> Document {
        let mut map = HashMap::new();
        let mut i = start_from;

        loop {
            let element_type = self.bytes.get(i);
            let Some(element_type) = element_type else {
                break;
            };
            if *element_type == 0x00 {
                break;
            }

            let name = self.parse_cstring(i + 1);
            i += 1 + name.len() + 1;

            let value = match element_type {
                // Double
                0x01 => {
                    let value = self.parse_double(i);
                    i += 8;
                    Value::Double(value)
                }
                // String
                0x02 => {
                    let (value, size) = self.parse_string(i);
                    i += size + 4;
                    Value::String(value)
                }
                // Embedded Document
                0x03 => {
                    let size = i32::from_le_bytes(
                        self.bytes[i..i + 4]
                            .try_into()
                            .expect("message is well formed"),
                    ) as usize;
                    let start = i + 4;
                    let value = self.parse_document(start);
                    i += size;
                    Value::Document(value)
                }
                // Array
                0x04 => {
                    let size = i32::from_le_bytes(
                        self.bytes[i..i + 4]
                            .try_into()
                            .expect("message is well formed"),
                    ) as usize;
                    let start = i + 4;
                    let value = self.parse_document(start);
                    i += size;
                    Value::Array(Array::from_document(value))
                }
                // Binary
                0x05 => {
                    let value = self.parse_binary(i);
                    i += value.len() + 1;
                    Value::Binary(value)
                }
                0x06 => {
                    // Undefined
                    i += 1;
                    Value::Undefined
                }
                0x07 => {
                    // ObjectId
                    let value = self.parse_object_id(i);
                    i += 12;
                    Value::ObjectId(value)
                }
                0x08 => {
                    // Boolean
                    let value = self.parse_boolean(i);
                    i += 1;
                    Value::Boolean(value)
                }
                0x09 => {
                    // UTCDateTime
                    let value = self.parse_utc_date_time(i);
                    i += 8;
                    Value::UtcDateTime(value)
                }
                0x0A => {
                    // Null
                    Value::Null
                }
                0x0B => {
                    // Regex
                    let value = self.parse_regex(i);
                    i += value.0.len() + 1 + value.1.len() + 1;
                    Value::Regex(value.0, value.1)
                }
                0x0C => {
                    // DBPointer
                    let value = self.parse_db_pointer(i);
                    i += value.0.len() + 1 + 12;
                    Value::DBPointer(value.0, value.1)
                }
                0x0D => {
                    // JavaScriptCode
                    let value = self.parse_java_script_code(i);
                    i += 1 + name.len() + 1 + value.len() + 1;
                    Value::JavaScriptCode(value)
                }
                0x0E => {
                    // Symbol
                    let value = self.parse_symbol(i);
                    i += value.len() + 1;
                    Value::Symbol(value)
                }
                0x0F => {
                    // JavaScriptCodeWithScope
                    // let value = self.parse_java_script_code_with_scope(i + 1 + name.len() + 1);
                    // println!("name: {:?}, value: {:?}", name, value);
                    // i += 1 + name.len() + 1 + value.0.len() + 1 + value.1.len() + 1;
                    // Value::JavaScriptCodeWithScope(value.0, value.1)
                    todo!()
                }
                0x10 => {
                    // Int32
                    let value = self.parse_int32(i);
                    i += 4;
                    Value::Int32(value)
                }
                0x11 => {
                    // Timestamp
                    let value = self.parse_timestamp(i);
                    i += 8;
                    Value::Timestamp(value)
                }
                0x12 => {
                    // Int64
                    let value = self.parse_int64(i);
                    i += 8;
                    Value::Int64(value)
                }
                0x13 => {
                    // Decimal128
                    // let value = self.parse_decimal128(i + 1 + name.len() + 1);
                    // println!("name: {:?}, value: {:?}", name, value);
                    // i += 1 + name.len() + 1 + 16;
                    // Value::Decimal128(value)
                    todo!()
                }
                0xFF => {
                    // MinKey
                    i += 1 + name.len() + 1;
                    Value::MinKey
                }
                0x7F => {
                    // MaxKey
                    i += 1 + name.len() + 1;
                    Value::MaxKey
                }
                _ => {
                    panic!("unknown element type: {:x?}", element_type);
                }
            };
            map.insert(name, value);

            if i >= self.len() as usize {
                break;
            }
        }
        Document(map)
    }

    pub fn parse_string(&self, i: usize) -> (String, usize) {
        let size = i32::from_le_bytes(
            self.bytes[i..i + 4]
                .try_into()
                .expect("message is well formed"),
        ) as usize;

        // last byte is null terminator
        let str = String::from_utf8(self.bytes[i + 4..i + 4 + size - 1].to_vec()).unwrap();
        (str, size)
    }

    pub fn parse_cstring(&self, i: usize) -> String {
        let mut name = String::new();
        let mut i = i;
        while self.bytes[i] != 0 {
            name.push(self.bytes[i] as char);
            i += 1;
        }
        name
    }

    pub fn parse_double(&self, i: usize) -> f64 {
        f64::from_le_bytes(
            self.bytes[i..i + 8]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn parse_binary(&self, i: usize) -> Vec<u8> {
        let size = i32::from_le_bytes(
            self.bytes[i..i + 4]
                .try_into()
                .expect("message is well formed"),
        ) as usize;

        self.bytes[i + 4..i + 4 + size].to_vec()
    }

    pub fn parse_object_id(&self, i: usize) -> Vec<u8> {
        self.bytes[i..i + 12].to_vec()
    }

    pub fn parse_boolean(&self, i: usize) -> bool {
        self.bytes[i] == 0x01
    }

    pub fn parse_utc_date_time(&self, i: usize) -> u64 {
        u64::from_le_bytes(
            self.bytes[i..i + 8]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn parse_regex(&self, i: usize) -> (String, String) {
        let pattern = self.parse_cstring(i);
        let options = self.parse_cstring(i + pattern.len() + 1);
        (pattern, options)
    }

    pub fn parse_db_pointer(&self, i: usize) -> (String, Vec<u8>) {
        let collection = self.parse_cstring(i);
        let id = self.bytes[i + collection.len() + 1..i + collection.len() + 1 + 12].to_vec();
        (collection, id)
    }

    pub fn parse_java_script_code(&self, i: usize) -> String {
        self.parse_cstring(i)
    }

    pub fn parse_symbol(&self, i: usize) -> String {
        self.parse_cstring(i)
    }

    pub fn parse_java_script_code_with_scope(&self, i: usize) -> (String, Document) {
        // let code = self.parse_java_script_code(i);
        // let scope = self.parse_document(i + code.len() + 1);
        // (code, scope)
        todo!()
    }

    pub fn parse_int32(&self, i: usize) -> i32 {
        i32::from_le_bytes(
            self.bytes[i..i + 4]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn parse_timestamp(&self, i: usize) -> u64 {
        u64::from_le_bytes(
            self.bytes[i..i + 8]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn parse_int64(&self, i: usize) -> i64 {
        i64::from_le_bytes(
            self.bytes[i..i + 8]
                .try_into()
                .expect("message is well formed"),
        )
    }

    // pub fn parse_decimal128(&self, i: usize) -> Decimal128 {
    //     todo!()
    // }

    pub fn parse_min_key(&self, i: usize) {
        todo!()
    }

    pub fn parse_max_key(&self, i: usize) {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_bson() {
        let data = [
            0x16, 0x00, 0x00, 0x00, // total document size
            0x02, // 0x02 = type String
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, // field name "hello"
            0x06, 0x00, 0x00, 0x00, 0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, // field value "world"
            0x00, // 0x00 = type EOO ('end of object')
        ];
        let doc = Bson::new(&data).parse();
        println!("doc: {:#?}", doc);
    }

    #[test]
    fn test_nested_bson() {
        let data = [
            0x31, 0x00, 0x00, 0x00, // total document size
            0x03, // 0x03 = type Embedded Document
            0x6e, 0x65, 0x73, 0x74, 0x65, 0x64, 0x00, // field name "nested"
            0x1c, 0x00, 0x00, 0x00, // size of the nested document
            0x02, // 0x02 = type String
            0x6e, 0x61, 0x6d, 0x65, 0x00, // field name "name"
            0x05, 0x00, 0x00, 0x00, // string size
            0x42, 0x53, 0x4f, 0x4e, 0x00, // field value "BSON"
            0x10, // 0x10 = type 32-bit Integer
            0x61, 0x67, 0x65, 0x00, // field name "age"
            0x1e, 0x00, 0x00, 0x00, // field value 30
            0x00, // 0x00 = type EOO (end of object) for nested document
            0x00, // 0x00 = type EOO (end of object) for the outer document
        ];
        let doc = Bson::new(&data).parse();
        println!("doc: {:#?}", doc);
    }
}
