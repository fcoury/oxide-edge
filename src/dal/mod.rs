use std::path::Path;

use duckdb::DuckdbConnectionManager;
use r2d2::{ManageConnection, Pool};
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use r2d2_sqlite::SqliteConnectionManager;

pub struct ConnectionProvider<M>
where
    M: ManageConnection,
{
    pool: Pool<M>,
}

impl<M> ConnectionProvider<M>
where
    M: ManageConnection,
{
    pub fn conn(&self) -> r2d2::PooledConnection<M> {
        self.pool.get().unwrap()
    }
}

impl ConnectionProvider<DuckdbConnectionManager> {
    pub fn new(db_file: Option<&Path>) -> Self {
        let manager = match db_file {
            Some(path) => DuckdbConnectionManager::file(path).unwrap(),
            None => DuckdbConnectionManager::memory().unwrap(),
        };

        let pool = Pool::new(manager).unwrap();
        Self { pool }
    }
}

impl ConnectionProvider<SqliteConnectionManager> {
    pub fn new(db_file: Option<&Path>) -> Self {
        let manager = match db_file {
            Some(path) => SqliteConnectionManager::file(path),
            None => SqliteConnectionManager::memory(),
        };

        let pool = Pool::new(manager).unwrap();
        Self { pool }
    }
}

impl ConnectionProvider<PostgresConnectionManager<NoTls>> {
    pub fn new() -> Self {
        let manager = PostgresConnectionManager::new(
            "host=localhost user=postgres password=postgres dbname=postgres"
                .parse()
                .unwrap(),
            NoTls,
        );
        let pool = Pool::new(manager).unwrap();
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_connection() {
        let provider = ConnectionProvider::<DuckdbConnectionManager>::new(None);
        let conn = provider.conn();
        println!("{:?}", conn);

        let provider = ConnectionProvider::<SqliteConnectionManager>::new(None);
        let conn = provider.conn();
        println!("{:?}", conn);

        let provider = ConnectionProvider::<PostgresConnectionManager<NoTls>>::new();
        let conn = provider.conn();
    }
}
