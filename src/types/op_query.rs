use std::fmt;

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

    pub fn query(&self) -> Vec<u8> {
        let i = 20 + self.full_collection_name().len() + 1 + 8;
        println!("bytes: {:?}", self.bytes[i..i + 4].to_vec());
        let size = i32::from_le_bytes(
            self.bytes[i..i + 4]
                .try_into()
                .expect("message is well formed"),
        ) as usize;
        println!("size: {}", size);

        vec![]
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
