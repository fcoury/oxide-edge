use oxide::command::listdatabases;

mod common;

#[test]
fn test_list_databases_empty() {
    let db_conn = common::get_conn().unwrap();
    let op_code = common::get_msg_from_fixture("request_listdatabases").unwrap();
    let response = listdatabases::run(op_code, db_conn).unwrap();
    assert_eq!(response.get_i32("ok").unwrap(), 1);
    assert_eq!(response.get_array("databases").unwrap().len(), 0);
}

#[test]
fn test_list_databases_non_empty() {
    let db_conn = common::get_conn().unwrap();
    db_conn.execute("CREATE SCHEMA test", []).unwrap();

    let op_code = common::get_msg_from_fixture("request_listdatabases").unwrap();
    let response = listdatabases::run(op_code, db_conn).unwrap();
    let databases = response.get_array("databases").unwrap();
    assert_eq!(response.get_i32("ok").unwrap(), 1);
    assert_eq!(databases.len(), 1);
    assert_eq!(
        databases[0].as_document().unwrap().get_str("name").unwrap(),
        "test"
    );
}
