use bson::doc;
use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::OpCode;
use r2d2::PooledConnection;

use super::CommandResult;

pub fn run(op_code: OpCode, db_conn: PooledConnection<DuckdbConnectionManager>) -> CommandResult {
    let doc = op_code.document();
    let db = doc.get_str("$db").unwrap();

    let mut stmt = db_conn.prepare(
        "SELECT table_name FROM duckdb_tables WHERE schema_name = ? ORDER BY table_oid;",
    )?;
    let tables = stmt
        .query_map([db], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    let collections = tables
        .iter()
        .map(|table| {
            doc! {
                "name": table,
                "type": "collection"
            }
        })
        .collect::<Vec<_>>();

    Ok(doc! {
        "ok": 1,
        "cursor": {
            "firstBatch": bson::to_bson(&collections)?,
            "id": 0i64,
            "ns": format!("{db}.$cmd.listCollections"),
        }
    })
}
