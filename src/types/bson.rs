use std::collections::HashMap;

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
            let element_type = self.bytes[i];
            let name = self.parse_cstring(i + 1);

            println!("element_type: {:#x?}", element_type);

            let value = match element_type {
                0x00 => break,
                0x01 => {
                    let value = self.parse_double(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + 8;
                    Value::Double(value)
                }
                0x02 => {
                    let len = i32::from_le_bytes(
                        self.bytes[i + 1 + name.len() + 1..i + 1 + name.len() + 1 + 4]
                            .try_into()
                            .expect("message is well formed"),
                    ) as usize;
                    println!("str len: {}", len);
                    let value = self.parse_string(i + 1 + name.len() + 1);
                    println!("str: {}", value);
                    i += 1 + name.len() + 1 + len + 4;
                    println!("next byte: {:#x?}", self.bytes[i]);
                    Value::String(value)
                }
                0x03 => {
                    let len = i32::from_le_bytes(
                        self.bytes[i + 1 + name.len() + 1..i + 1 + name.len() + 1 + 4]
                            .try_into()
                            .expect("message is well formed"),
                    ) as usize;
                    let value = self.parse_document(i + 1 + name.len() + 1 + 4);
                    i += 1 + name.len() + 1 + len + 1 + 4;
                    Value::Document(value)
                }
                0x04 => {
                    // let len = i32::from_le_bytes(
                    //     self.bytes[i + 1 + name.len() + 1..i + 1 + name.len() + 1 + 4]
                    //         .try_into()
                    //         .expect("message is well formed"),
                    // ) as usize;
                    // let value = self.parse_document(i + 1 + name.len() + 1 + 4);
                    // println!("name: {:?}, value: {:?}", name, value);
                    // i += 1 + name.len() + 1 + len + 1 + 4;
                    todo!("Array")
                }
                0x05 => {
                    let value = self.parse_binary(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + value.len() + 1;
                    Value::Binary(value)
                }
                0x06 => {
                    println!("name: {:?}", name);
                    i += 1 + name.len() + 1;
                    Value::Undefined
                }
                0x07 => {
                    let value = self.parse_object_id(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + 12;
                    Value::ObjectId(value)
                }
                0x08 => {
                    let value = self.parse_boolean(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + 1;
                    if value {
                        Value::True
                    } else {
                        Value::False
                    }
                }
                0x09 => {
                    let value = self.parse_utc_date_time(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + 8;
                    Value::UtcDateTime(value)
                }
                0x0A => {
                    println!("name: {:?}", name);
                    i += 1 + name.len() + 1;
                    Value::Null
                }
                0x0B => {
                    let value = self.parse_regex(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + value.0.len() + 1 + value.1.len() + 1;
                    Value::Regex(value.0, value.1)
                }
                0x0C => {
                    let value = self.parse_db_pointer(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + value.0.len() + 1 + 12;
                    Value::DBPointer(value.0, value.1)
                }
                0x0D => {
                    let value = self.parse_java_script_code(i + 1 + name.len() + 1);
                    println!("name: {:?}, value: {:?}", name, value);
                    i += 1 + name.len() + 1 + value.len() + 1;
                    Value::JavaScriptCode(value)
                }
                0x0E => {
                    let value = self.parse_symbol(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + value.len() + 1;
                    Value::Symbol(value)
                }
                0x0F => {
                    // let value = self.parse_java_script_code_with_scope(i + 1 + name.len() + 1);
                    // println!("name: {:?}, value: {:?}", name, value);
                    // i += 1 + name.len() + 1 + value.0.len() + 1 + value.1.len() + 1;
                    // Value::JavaScriptCodeWithScope(value.0, value.1)
                    todo!()
                }
                0x10 => {
                    let value = self.parse_int32(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + 4;
                    Value::Int32(value)
                }
                0x11 => {
                    let value = self.parse_timestamp(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + 8;
                    Value::Timestamp(value)
                }
                0x12 => {
                    let value = self.parse_int64(i + 1 + name.len() + 1);
                    i += 1 + name.len() + 1 + 8;
                    Value::Int64(value)
                }
                0x13 => {
                    // let value = self.parse_decimal128(i + 1 + name.len() + 1);
                    // println!("name: {:?}, value: {:?}", name, value);
                    // i += 1 + name.len() + 1 + 16;
                    // Value::Decimal128(value)
                    todo!()
                }
                0xFF => {
                    println!("name: {:?}", name);
                    i += 1 + name.len() + 1;
                    Value::MinKey
                }
                0x7F => {
                    println!("name: {:?}", name);
                    i += 1 + name.len() + 1;
                    Value::MaxKey
                }
                _ => {
                    panic!("unknown element type: {:x?}", element_type);
                }
            };
            println!("name: {:?}, value: {:?}", name, value);
            map.insert(name, value);

            if i >= self.len() as usize {
                break;
            }
        }
        Document(map)
    }

    pub fn parse_string(&self, i: usize) -> String {
        let len = i32::from_le_bytes(
            self.bytes[i..i + 4]
                .try_into()
                .expect("message is well formed"),
        ) as usize;

        // last byte is null terminator
        String::from_utf8(self.bytes[i + 4..i + 4 + len - 1].to_vec()).unwrap()
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

    // pub fn parse_document(&self, i: usize) -> Document {
    //     let size = i32::from_le_bytes(
    //         self.bytes[i..i + 4]
    //             .try_into()
    //             .expect("message is well formed"),
    //     ) as usize;
    //
    //     Document::new(&self.bytes[i..i + size])
    // }

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
        self.bytes[i] == 0x0
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
