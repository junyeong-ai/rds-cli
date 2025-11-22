use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use tokio_postgres::{Client, NoTls};

use super::{Database, QueryResult};
use crate::cache::{ColumnMetadata, ForeignKeyRelationship, SchemaCache, TableMetadata};
use crate::config::DatabaseProfile;

pub struct PostgresDatabase {
    client: Option<Client>,
}

impl Default for PostgresDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl PostgresDatabase {
    pub fn new() -> Self {
        Self { client: None }
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn connect(&mut self, profile: &DatabaseProfile) -> Result<()> {
        let config = format!(
            "host={} port={} user={} password={} dbname={}",
            profile.host, profile.port, profile.user, profile.password, profile.database
        );

        let (client, connection) = tokio_postgres::connect(&config, NoTls)
            .await
            .context("Failed to connect to PostgreSQL")?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });

        self.client = Some(client);
        Ok(())
    }

    async fn extract_schema(&self, profile: &DatabaseProfile) -> Result<SchemaCache> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to database"))?;

        let schema_name = profile.schema.as_deref().unwrap_or("public");

        let query = "
            SELECT
                c.table_name,
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                CASE
                    WHEN pk.column_name IS NOT NULL THEN true
                    ELSE false
                END as is_primary_key
            FROM information_schema.columns c
            LEFT JOIN (
                SELECT kcu.table_name, kcu.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                WHERE tc.constraint_type = 'PRIMARY KEY'
                    AND tc.table_schema = $1
            ) pk ON c.table_name = pk.table_name
                AND c.column_name = pk.column_name
            WHERE c.table_schema = $1
            ORDER BY c.table_name, c.ordinal_position
        ";

        let rows = client.query(query, &[&schema_name]).await?;

        let mut tables: HashMap<String, TableMetadata> = HashMap::new();

        for row in rows {
            let table_name: String = row.get(0);
            let column_name: String = row.get(1);
            let data_type: String = row.get(2);
            let is_nullable: String = row.get(3);
            let column_default: Option<String> = row.get(4);
            let is_primary_key: bool = row.get(5);

            let column = ColumnMetadata {
                name: column_name.clone(),
                data_type,
                nullable: is_nullable == "YES",
                default_value: column_default,
                is_primary_key,
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

            if is_primary_key {
                table.primary_key.push(column_name.clone());
            }

            table.columns.push(column);
        }

        let fk_query = "
            SELECT
                tc.constraint_name,
                tc.table_name as source_table,
                kcu.column_name as source_column,
                ccu.table_name as target_table,
                ccu.column_name as target_column
            FROM information_schema.table_constraints tc
            JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = $1
        ";

        let fk_rows = client.query(fk_query, &[&schema_name]).await?;

        for row in fk_rows {
            let constraint_name: String = row.get(0);
            let source_table: String = row.get(1);
            let source_column: String = row.get(2);
            let target_table: String = row.get(3);
            let target_column: String = row.get(4);

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
            database_type: "postgresql".to_string(),
            tables,
        })
    }

    async fn execute_query(&self, sql: &str, timeout_secs: u64) -> Result<QueryResult> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to database"))?;

        client
            .execute(
                &format!("SET statement_timeout = {}", timeout_secs * 1000),
                &[],
            )
            .await?;

        let rows = client.query(sql, &[]).await?;

        let columns = if !rows.is_empty() {
            rows[0]
                .columns()
                .iter()
                .map(|col| col.name().to_string())
                .collect()
        } else {
            Vec::new()
        };

        let result_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| {
                        use tokio_postgres::types::Type;

                        let col_type = row.columns()[i].type_();

                        match *col_type {
                            Type::BOOL => row
                                .try_get::<_, Option<bool>>(i)
                                .ok()
                                .flatten()
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "NULL".to_string()),
                            Type::INT2 | Type::INT4 => row
                                .try_get::<_, Option<i32>>(i)
                                .ok()
                                .flatten()
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "NULL".to_string()),
                            Type::INT8 => row
                                .try_get::<_, Option<i64>>(i)
                                .ok()
                                .flatten()
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "NULL".to_string()),
                            Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => row
                                .try_get::<_, Option<String>>(i)
                                .ok()
                                .flatten()
                                .unwrap_or_else(|| "NULL".to_string()),
                            Type::UUID => row
                                .try_get::<_, Option<uuid::Uuid>>(i)
                                .ok()
                                .flatten()
                                .map(|v| v.to_string())
                                .unwrap_or_else(|| "NULL".to_string()),
                            Type::TIMESTAMPTZ | Type::TIMESTAMP => row
                                .try_get::<_, Option<chrono::NaiveDateTime>>(i)
                                .ok()
                                .flatten()
                                .map(|v| v.to_string())
                                .or_else(|| {
                                    row.try_get::<_, Option<chrono::DateTime<chrono::Utc>>>(i)
                                        .ok()
                                        .flatten()
                                        .map(|v| v.to_string())
                                })
                                .unwrap_or_else(|| "NULL".to_string()),
                            _ => row
                                .try_get::<_, Option<String>>(i)
                                .ok()
                                .flatten()
                                .unwrap_or_else(|| format!("({})", col_type.name())),
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
        "postgresql"
    }
}
