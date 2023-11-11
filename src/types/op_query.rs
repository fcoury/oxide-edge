use std::{collections::HashMap, fmt};

use crate::types::Bson;

use super::{Document, Value};

pub struct OpQuery<'a> {
    pub bytes: &'a [u8],
}

impl<'a> OpQuery<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    pub fn flags(&self) -> i32 {
        i32::from_le_bytes(
            self.bytes[16..20]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn full_collection_name(&self) -> String {
        let mut full_collection_name = String::new();
        let mut i = 20;
        while self.bytes[i] != 0 {
            full_collection_name.push(self.bytes[i] as char);
            i += 1;
        }
        full_collection_name
    }

    pub fn number_to_skip(&self) -> i32 {
        let i = 20 + self.full_collection_name().len() + 1;
        i32::from_le_bytes(
            self.bytes[i..i + 4]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn number_to_return(&self) -> i32 {
        let i = 20 + self.full_collection_name().len() + 1 + 4;
        i32::from_le_bytes(
            self.bytes[i..i + 4]
                .try_into()
                .expect("message is well formed"),
        )
    }

    pub fn query(&self) -> Document {
        let i = 20 + self.full_collection_name().len() + 1 + 8;
        let bson = Bson::from_bytes(&self.bytes[i..]);
        let result = bson.parse();
        result
    }
}

impl fmt::Debug for OpQuery<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpQuery")
            .field("flags", &self.flags())
            .field("full_collection_name", &self.full_collection_name())
            .field("number_to_skip", &self.number_to_skip())
            .field("number_to_return", &self.number_to_return())
            .field("query", &self.query())
            .finish()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mongosh_connect_message() {
        let data: [u8; 344] = [
            88, 1, 0, 0, // total document size
            1, 0, 0, 0, // request id
            0, 0, 0, 0, // response to
            212, 7, 0, 0, // op code
            0, 0, 0, 0, // flags
            97, 100, 109, 105, 110, 46, 36, 99, 109, 100, 0, // collection name (admin.$cmd)
            0, 0, 0, 0, // number to skip
            255, 255, 255, 255, // number to return
            // document
            49, 1, 0, 0, // document size
            // field { ismaster: 1 }
            16, // type 16 (0x10) - int32
            105, 115, 109, 97, 115, 116, 101, 114, 0, // field name (ismaster)
            1, 0, 0, 0, // value (1)
            // field { helloOk: true }
            8, // type 8 (0x08) - boolean
            104, 101, 108, 108, 111, 79, 107, 0, // field name (helloOk)
            1, // value (true)
            // field
            3, // type 3 (0x03) - document
            99, 108, 105, 101, 110, 116, 0, // field name (client)
            238, 0, 0, 0, // document size (238)
            // nested field
            3, // type 3 (0x03) - document
            97, 112, 112, 108, 105, 99, 97, 116, 105, 111, 110, 0, // field name (application)
            29, 0, 0, 0, // document size (29)
            // nested field
            2, // type 2 (0x02) - string
            110, 97, 109, 101, 0, // field name (name)
            14, 0, 0, 0, // string size (14)
            109, 111, 110, 103, 111, 115, 104, 32, 50, 46, 48, 46, 50,
            0, // string value (mongosh 2.0.2)
            0, // end of document
            3, // type 3 (0x03) - document
            100, 114, 105, 118, 101, 114, 0, // field name (driver)
            55, 0, 0, 0, // document size (55)
            2, // type 2 (0x02) - string
            110, 97, 109, 101, 0, // field name (name)
            15, 0, 0, 0, // string size (15)
            110, 111, 100, 101, 106, 115, 124, 109, 111, 110, 103, 111, 115, 104,
            0, // string value (nodejs|mongosh)
            2, // type 2 (0x02) - string
            118, 101, 114, 115, 105, 111, 110, 0, // field name (version)
            12, 0, 0, 0, // string size (12)
            54, 46, 48, 46, 48, 124, 50, 46, 48, 46, 50, 0, // string value (6.0.0|2.0.2)
            0, // end of document
            2, // type 2 (0x02) - string
            112, 108, 97, 116, 102, 111, 114, 109, 0, // field name (platform)
            20, 0, 0, 0, // string size (20)
            78, 111, 100, 101, 46, 106, 115, 32, 118, 50, 48, 46, 56, 46, 49, 44, 32, 76, 69,
            0, // string value (Node.js v20.8.1, LE)
            3, // type 3 (0x03) - document
            111, 115, 0, // field name (os)
            90, 0, 0, 0, // document size (90)
            2, // type 2 (0x02) - string
            110, 97, 109, 101, 0, // field name (name)
            6, 0, 0, 0, // string size (6)
            108, 105, 110, 117, 120, 0, // string value (linux)
            2, // type 2 (0x02) - string
            97, 114, 99, 104, 105, 116, 101, 99, 116, 117, 114, 101,
            0, // field name (architecture)
            4, 0, 0, 0, // string size (4)
            120, 54, 52, 0, // string value (x64)
            2, // type 2 (0x02) - string
            118, 101, 114, 115, 105, 111, 110, 0, // field name (version)
            18, 0, 0, 0, // string size (18)
            53, 46, 49, 53, 46, 48, 45, 56, 55, 45, 103, 101, 110, 101, 114, 105, 99,
            0, // string value (5.15.0-87-generic)
            2, // type 2 (0x02) - string
            116, 121, 112, 101, 0, // field name (type)
            6, 0, 0, 0, // string size (6)
            76, 105, 110, 117, 120, 0, // string value (Linux)
            0, // end of document
            0, // end of document
            4, // type 4 (0x04) - array
            99, 111, 109, 112, 114, 101, 115, 115, 105, 111, 110, 0, 17, 0, 0, 0, 2, 48, 0, 5, 0,
            0, 0, 110, 111, 110, 101, 0, 0, 0,
        ];
        let op_query = OpQuery::new(&data);

        println!("doc: {:#?}", op_query);
    }
}
