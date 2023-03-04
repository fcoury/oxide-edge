use bson::doc;
use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::OpCode;
use r2d2::PooledConnection;

use super::CommandResult;

pub fn run(op_code: OpCode, db_conn: PooledConnection<DuckdbConnectionManager>) -> CommandResult {
    let doc = op_code.document();
    let db = doc.get_str("$db")?;
    let coll = doc.get_str("find")?;
    let _filter = doc.get_document("filter")?;

    db_conn.execute_batch(&format!(
        r#"
        CREATE SCHEMA IF NOT EXISTS {db};
        CREATE TABLE IF NOT EXISTS {db}.{coll} (data JSON);
        "#
    ))?;

    // TODO: filters
    let mut stmt = db_conn.prepare(&format!("SELECT data FROM {db}.{coll};"))?;
    let data = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<serde_json::Value>, _>>()?;

    Ok(doc! {
        "ok": 1,
        "cursor": {
            "firstBatch": bson::to_bson(&data)?,
            "id": 0i64,
            "ns": format!("{db}.{coll}"),
        }
    })
}
