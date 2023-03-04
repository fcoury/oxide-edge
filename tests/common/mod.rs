#![allow(dead_code)]
use std::{error::Error, fs};

use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::{parse, OpCode};
use r2d2::PooledConnection;

pub fn get_pool() -> Result<r2d2::Pool<DuckdbConnectionManager>, Box<dyn Error>> {
    let manager = DuckdbConnectionManager::memory()?;
    let pool = r2d2::Pool::new(manager)?;

    pool.get()?.execute_batch("INSTALL 'json'; LOAD 'json';")?;

    Ok(pool)
}

pub fn get_conn() -> Result<PooledConnection<DuckdbConnectionManager>, Box<dyn Error>> {
    let pool = get_pool()?;
    let db_conn = pool.get()?;

    Ok(db_conn)
}

pub fn query_one<T: duckdb::types::FromSql>(
    pool: r2d2::Pool<DuckdbConnectionManager>,
    query: &str,
) -> Result<Vec<T>, Box<dyn Error>> {
    let db_conn = pool.get()?;
    let mut stmt = db_conn.prepare(query)?;
    let res = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<T>, _>>()?;

    Ok(res)
}

pub fn get_msg_from_fixture(fixture: &str) -> Result<OpCode, Box<dyn Error>> {
    Ok(parse(fs::read(format!("tests/fixtures/{fixture}.bin"))?)?)
}
