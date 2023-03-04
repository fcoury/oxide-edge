use oxide::command::insert;

mod common;

#[test]
fn test_insert_simple() {
    let pool = common::get_pool().unwrap();
    let op_code = common::get_msg_from_fixture("request_insert_simple").unwrap();
    let response = insert::run(op_code, pool.get().unwrap()).unwrap();
    assert_eq!(response.get_i32("ok").unwrap(), 1);

    let schemas =
        common::query_one::<String>(pool, "SELECT schema_name FROM duckdb_schemas").unwrap();
    assert_eq!(schemas.len(), 1);
    assert_eq!(schemas[0], "test");
}
