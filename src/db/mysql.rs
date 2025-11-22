use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use mysql_async::prelude::*;
use mysql_async::{OptsBuilder, Pool, Row};
use std::collections::HashMap;

use super::{Database, QueryResult};
use crate::cache::{ColumnMetadata, ForeignKeyRelationship, SchemaCache, TableMetadata};
use crate::config::DatabaseProfile;

pub struct MySqlDatabase {
    pool: Option<Pool>,
}

impl Default for MySqlDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl MySqlDatabase {
    pub fn new() -> Self {
        Self { pool: None }
    }
}

#[async_trait]
impl Database for MySqlDatabase {
    async fn connect(&mut self, profile: &DatabaseProfile) -> Result<()> {
        let opts = OptsBuilder::default()
            .ip_or_hostname(&profile.host)
            .tcp_port(profile.port)
            .user(Some(&profile.user))
            .pass(Some(&profile.password))
            .db_name(Some(&profile.database));

        let pool = Pool::new(opts);
        self.pool = Some(pool);
        Ok(())
    }

    async fn extract_schema(&self, profile: &DatabaseProfile) -> Result<SchemaCache> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to database"))?;

        let mut conn = pool.get_conn().await?;

        let query = format!(
            "
            SELECT
                c.TABLE_NAME,
                c.COLUMN_NAME,
                c.DATA_TYPE,
                c.IS_NULLABLE,
                c.COLUMN_DEFAULT,
                CASE
                    WHEN pk.COLUMN_NAME IS NOT NULL THEN 1
                    ELSE 0
                END as is_primary_key
            FROM information_schema.COLUMNS c
            LEFT JOIN (
                SELECT kcu.TABLE_NAME, kcu.COLUMN_NAME
                FROM information_schema.TABLE_CONSTRAINTS tc
                JOIN information_schema.KEY_COLUMN_USAGE kcu
                    ON tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME
                    AND tc.TABLE_SCHEMA = kcu.TABLE_SCHEMA
                WHERE tc.CONSTRAINT_TYPE = 'PRIMARY KEY'
                    AND tc.TABLE_SCHEMA = '{}'
            ) pk ON c.TABLE_NAME = pk.TABLE_NAME
                AND c.COLUMN_NAME = pk.COLUMN_NAME
            WHERE c.TABLE_SCHEMA = '{}'
            ORDER BY c.TABLE_NAME, c.ORDINAL_POSITION
            ",
            profile.database, profile.database
        );

        let rows: Vec<Row> = conn.query(query).await?;

        let mut tables: HashMap<String, TableMetadata> = HashMap::new();

        for row in rows {
            let table_name: String = row.get(0).unwrap();
            let column_name: String = row.get(1).unwrap();
            let data_type: String = row.get(2).unwrap();
            let is_nullable: String = row.get(3).unwrap();
            let column_default: Option<String> = row.get(4);
            let is_primary_key: i32 = row.get(5).unwrap();

            let column = ColumnMetadata {
                name: column_name.clone(),
                data_type,
                nullable: is_nullable == "YES",
                default_value: column_default,
                is_primary_key: is_primary_key == 1,
                is_foreign_key: false,
            };

            let table = tables
                .entry(table_name.clone())
                .or_insert_with(|| TableMetadata {
                    name: table_name.clone(),
                    columns: Vec::new(),
                    primary_key: Vec::new(),
                    foreign_keys: Vec::new(),
                    referenced_by: Vec::new(),
                });

            if is_primary_key == 1 {
                table.primary_key.push(column_name.clone());
            }

            table.columns.push(column);
        }

        let fk_query = format!(
            "
            SELECT
                kcu.CONSTRAINT_NAME,
                kcu.TABLE_NAME as source_table,
                kcu.COLUMN_NAME as source_column,
                kcu.REFERENCED_TABLE_NAME as target_table,
                kcu.REFERENCED_COLUMN_NAME as target_column
            FROM information_schema.KEY_COLUMN_USAGE kcu
            WHERE kcu.REFERENCED_TABLE_SCHEMA = '{}'
                AND kcu.REFERENCED_TABLE_NAME IS NOT NULL
            ",
            profile.database
        );

        let fk_rows: Vec<Row> = conn.query(fk_query).await?;

        for row in fk_rows {
            let constraint_name: String = row.get(0).unwrap();
            let source_table: String = row.get(1).unwrap();
            let source_column: String = row.get(2).unwrap();
            let target_table: String = row.get(3).unwrap();
            let target_column: String = row.get(4).unwrap();

            let fk = ForeignKeyRelationship {
                constraint_name,
                source_table: source_table.clone(),
                source_column: source_column.clone(),
                target_table,
                target_column,
            };

            if let Some(table) = tables.get_mut(&source_table) {
                table.foreign_keys.push(fk.clone());

                for col in &mut table.columns {
                    if col.name == source_column {
                        col.is_foreign_key = true;
                    }
                }
            }
        }

        Ok(SchemaCache {
            cached_at: Utc::now(),
            profile_name: profile.database.clone(),
            database_type: "mysql".to_string(),
            tables,
        })
    }

    async fn execute_query(&self, sql: &str, timeout_secs: u64) -> Result<QueryResult> {
        let pool = self
            .pool
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to database"))?;

        let mut conn = pool.get_conn().await?;

        conn.query_drop(format!("SET max_execution_time = {}", timeout_secs * 1000))
            .await?;

        let rows: Vec<Row> = conn.query(sql).await?;

        let columns = if !rows.is_empty() {
            rows[0]
                .columns()
                .iter()
                .map(|col| col.name_str().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let result_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| {
                        use mysql_async::Value;
                        match row.as_ref(i) {
                            Some(value) => match value {
                                Value::NULL => "NULL".to_string(),
                                Value::Bytes(b) => String::from_utf8_lossy(b).to_string(),
                                Value::Int(v) => v.to_string(),
                                Value::UInt(v) => v.to_string(),
                                Value::Float(v) => v.to_string(),
                                Value::Double(v) => v.to_string(),
                                Value::Date(y, m, d, h, min, s, ms) => {
                                    format!(
                                        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
                                        y, m, d, h, min, s, ms
                                    )
                                }
                                Value::Time(neg, d, h, m, s, ms) => {
                                    let sign = if *neg { "-" } else { "" };
                                    let total_hours = d * 24 + *h as u32;
                                    format!(
                                        "{}:{:02}:{:02}:{:02}.{:06}",
                                        sign, total_hours, m, s, ms
                                    )
                                }
                            },
                            None => "NULL".to_string(),
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(QueryResult {
            rows: result_rows.clone(),
            columns,
            rows_affected: result_rows.len(),
        })
    }

    fn db_type(&self) -> &str {
        "mysql"
    }
}
