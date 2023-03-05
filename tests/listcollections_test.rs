use oxide::command::listcollections;

mod common;

#[test]
fn test_list_collections_non_empty() {
    let db_conn = common::get_conn().unwrap();
    db_conn.execute("CREATE SCHEMA test", []).unwrap();
    db_conn
        .execute("CREATE TABLE test.test (data JSON)", [])
        .unwrap();
    db_conn
        .execute("CREATE TABLE test.other_test (data JSON)", [])
        .unwrap();

    let op_code = common::get_msg_from_fixture("request_listcollections").unwrap();
    let response = listcollections::run(op_code, db_conn).unwrap();
    assert_eq!(response.get_i32("ok").unwrap(), 1);
    let cursor = response.get_document("cursor").unwrap();
    let cols = cursor.get_array("firstBatch").unwrap();
    assert_eq!(cols.len(), 2);
    let col1 = cols[0].as_document().unwrap();
    let col2 = cols[1].as_document().unwrap();
    assert_eq!(col1.get_str("name").unwrap(), "test");
    assert_eq!(col2.get_str("name").unwrap(), "other_test");
}

#[test]
fn test_list_collections_empty() {
    let db_conn = common::get_conn().unwrap();
    let op_code = common::get_msg_from_fixture("request_listcollections").unwrap();
    let response = listcollections::run(op_code, db_conn).unwrap();
    assert_eq!(response.get_i32("ok").unwrap(), 1);
    let cursor = response.get_document("cursor").unwrap();
    let cols = cursor.get_array("firstBatch").unwrap();
    assert_eq!(cols.len(), 0);
}
