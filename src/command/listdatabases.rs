use bson::doc;
use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::OpCode;
use r2d2::PooledConnection;

use super::CommandResult;

pub fn run(op_code: OpCode, db_conn: PooledConnection<DuckdbConnectionManager>) -> CommandResult {
    let doc = op_code.document();
    let name_only = doc.get_bool("nameOnly").unwrap_or(false);

    let mut stmt = db_conn.prepare("SELECT schema_name FROM duckdb_schemas;")?;
    let schemas = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;

    Ok(doc! {
        "databases": schemas.iter().map(|schema| {
            if name_only {
                doc! {
                    "name": schema
                }
            } else {
                doc! {
                    "name": schema,
                    "sizeOnDisk": 0,
                    "empty": false
                }
            }
        }).collect::<Vec<_>>(),
        "totalSize": 0,
        "ok": 1
    })
}
