use bson::doc;
use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::OpCode;
use r2d2::PooledConnection;

use super::CommandResult;

pub fn run(op_code: OpCode, db_conn: PooledConnection<DuckdbConnectionManager>) -> CommandResult {
    let doc = op_code.document();
    let db = doc.get_str("$db")?;
    let coll = doc.get_str("insert")?;
    let documents = doc.get_array("documents")?;

    db_conn.execute_batch(&format!(
        r#"
        CREATE SCHEMA IF NOT EXISTS {db};
        CREATE TABLE IF NOT EXISTS {db}.{coll} (json JSON);
        "#
    ))?;

    let mut stmt = db_conn.prepare(&format!("INSERT INTO {db}.{coll} VALUES (?)"))?;
    for doc in documents {
        let doc = doc.as_document().unwrap();
        let json = serde_json::to_string(doc)?;
        stmt.execute([json])?;
    }

    Ok(doc! {
        "ok": 1
    })
}
