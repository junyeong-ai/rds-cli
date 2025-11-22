use anyhow::Result;
use async_trait::async_trait;

use crate::cache::SchemaCache;
use crate::config::DatabaseProfile;

pub mod mysql;
pub mod postgres;

#[async_trait]
pub trait Database: Send + Sync {
    async fn connect(&mut self, profile: &DatabaseProfile) -> Result<()>;
    async fn extract_schema(&self, profile: &DatabaseProfile) -> Result<SchemaCache>;
    async fn execute_query(&self, sql: &str, timeout_secs: u64) -> Result<QueryResult>;
    fn db_type(&self) -> &str;
}

#[derive(Debug)]
pub struct QueryResult {
    pub rows: Vec<Vec<String>>,
    pub columns: Vec<String>,
    pub rows_affected: usize,
}

pub fn create_database(db_type: &str) -> Result<Box<dyn Database>> {
    match db_type {
        "postgresql" => Ok(Box::new(postgres::PostgresDatabase::new())),
        "mysql" => Ok(Box::new(mysql::MySqlDatabase::new())),
        _ => anyhow::bail!("Unsupported database type: {}", db_type),
    }
}
