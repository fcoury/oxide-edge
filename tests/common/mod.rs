use std::{error::Error, fs};

use duckdb::DuckdbConnectionManager;
use mongodb_wire_protocol_parser::{parse, OpCode};
use r2d2::PooledConnection;

pub fn get_conn() -> Result<PooledConnection<DuckdbConnectionManager>, Box<dyn Error>> {
    let manager = DuckdbConnectionManager::memory().unwrap();
    let pool = r2d2::Pool::new(manager).unwrap();
    let db_conn = pool.get().unwrap();

    Ok(db_conn)
}

pub fn get_msg_from_fixture(fixture: &str) -> Result<OpCode, Box<dyn Error>> {
    Ok(parse(fs::read(format!("tests/fixtures/{fixture}.bin"))?)?)
}
