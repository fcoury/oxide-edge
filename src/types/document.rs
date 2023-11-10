pub struct Document {
    bytes: Vec<u8>,
}

pub enum Value {
    Double(f64),                               // \x01
    String(String),                            // \x02
    Document(Document),                        // \x03
    Array(Vec<Value>),                         // \x04
    Binary(Vec<u8>),                           // \x05
    Undefined,                                 // \x06
    ObjectId(Vec<u8>),                         // \x07
    False,                                     // \x08
    True,                                      // \x08
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

pub struct Decimal128;
