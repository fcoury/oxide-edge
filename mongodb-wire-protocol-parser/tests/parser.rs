use std::fs;

use mongodb_wire_protocol_parser::{parse_header, parse_op_code, OpCode, Section};

#[test]
fn test_parse_header() {
    let input = fs::read("tests/fixtures/listCollections-request.bin").unwrap();
    let (_input, header) = parse_header(&input).unwrap();
    assert_eq!(header.message_length, 207);
    assert_eq!(header.request_id, 5);
    assert_eq!(header.response_to, 0);
    assert_eq!(header.op_code, 2013);
}

#[test]
fn test_parse_list_collection_op_msg() {
    let input = fs::read("tests/fixtures/listCollections-request.bin").unwrap();
    let (_input, op_msg) = parse_op_code(&input).unwrap();
    let OpCode::OpMsg(op_msg) = op_msg else {
        panic!("Expected OpMsg");
    };
    let Section::Body(section) = op_msg.sections.get(0).unwrap() else {
        panic!("Expected BodySection");
    };
    let doc = &section.payload;
    assert_eq!(doc.get_str("$db").unwrap(), "test");
}

#[test]
fn test_parse_insert_op_msg() {
    let input = fs::read("tests/fixtures/insert-request.bin").unwrap();
    let (_input, op_msg) = parse_op_code(&input).unwrap();
    let OpCode::OpMsg(op_msg) = op_msg else {
        panic!("Expected OpMsg");
    };
    let Section::Body(section) = op_msg.sections.get(0).unwrap() else {
        panic!("Expected BodySection");
    };
    let doc = &section.payload;
    assert_eq!(doc.get_str("$db").unwrap(), "test");
    assert_eq!(doc.get_str("insert").unwrap(), "test");
    assert_eq!(doc.get_array("documents").unwrap().len(), 1);
}

#[test]
fn test_parse_ping_op_msg() {
    let input = fs::read("tests/fixtures/ping-request.bin").unwrap();
    let (_input, op_msg) = parse_op_code(&input).unwrap();
    let OpCode::OpMsg(op_msg) = op_msg else {
        panic!("Expected OpMsg");
    };
    let Section::Body(section) = op_msg.sections.get(0).unwrap() else {
        panic!("Expected BodySection");
    };
    let doc = &section.payload;
    assert_eq!(doc.get_i32("ping").unwrap(), 1);
    assert_eq!(doc.get_str("$db").unwrap(), "admin");
}

#[test]
fn test_parse_is_master() {
    let input = fs::read("tests/fixtures/isMaster-request.bin").unwrap();
    let (_input, op_query) = parse_op_code(&input).unwrap();
    let OpCode::OpQuery(op_query) = op_query else {
        panic!("Expected OpQuery");
    };
    assert_eq!(op_query.query.get_i32("isMaster").unwrap(), 1);
    assert!(op_query.query.get_bool("helloOk").unwrap());
}

// #[test]
// fn test_multiple() {
//     // iterate through files in ../tokio-proxy/logs
//     for file in fs::read_dir("../tokio-proxy/logs")
//         .unwrap()
//         .filter_map(|e| e.ok())
//         .filter_map(|e| e.path().to_string_lossy().contains("request").then(|| e))
//         .map(|e| e.path())
//         .collect::<Vec<_>>()
//     {
//         println!("Parsing {:?}", file);
//         let input = fs::read(file).unwrap();
//         let (_input, op_code) = parse_op_code(&input).unwrap();
//         println!("{op_code:#?}");
//     }
// }
