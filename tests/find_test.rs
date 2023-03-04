use oxide::command::find;
use serde_json::json;

mod common;

#[test]
fn test_find_simple() {
    let pool = common::get_pool().unwrap();
    common::insert(&pool, "test", "test", json!({"a": 1 })).unwrap();

    let op_code = common::get_msg_from_fixture("request_find_simple").unwrap();
    let response = find::run(op_code, pool.get().unwrap()).unwrap();
    assert_eq!(response.get_i32("ok").unwrap(), 1);
    let cursor = response.get_document("cursor").unwrap();
    let first_batch = cursor.get_array("firstBatch").unwrap();
    assert_eq!(first_batch.len(), 1);
    let doc = first_batch[0].as_document().unwrap();
    assert_eq!(doc.get_i64("a").unwrap(), 1);
}

#[test]
fn test_find_missing_table() {
    let pool = common::get_pool().unwrap();
    let op_code = common::get_msg_from_fixture("request_find_simple").unwrap();
    let response = find::run(op_code, pool.get().unwrap()).unwrap();
    assert_eq!(response.get_i32("ok").unwrap(), 1);
    let cursor = response.get_document("cursor").unwrap();
    let first_batch = cursor.get_array("firstBatch").unwrap();
    assert_eq!(first_batch.len(), 0);
}
