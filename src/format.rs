use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use tabled::{Table, Tabled};

use crate::cache::{ColumnMetadata, ForeignKeyRelationship, TableMetadata};
use crate::config::SavedQuery;

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
    JsonPretty,
    Csv,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "json-pretty" | "pretty" => Ok(Self::JsonPretty),
            "csv" => Ok(Self::Csv),
            _ => anyhow::bail!(
                "Unknown format: {}. Available: table, json, json-pretty, csv",
                s
            ),
        }
    }
}

#[derive(Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub rows_affected: usize,
}

pub fn format_query_result(
    columns: &[String],
    rows: &[Vec<String>],
    rows_affected: usize,
    format: OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Table => {
            let mut output = String::new();
            output.push_str(&columns.join(" | "));
            output.push('\n');
            output.push_str(&"-".repeat(columns.len() * 20));
            output.push('\n');
            for row in rows {
                output.push_str(&row.join(" | "));
                output.push('\n');
            }
            output.push_str(&format!("\n{} rows returned", rows_affected));
            Ok(output)
        }
        OutputFormat::Json => {
            let result = QueryResult {
                columns: columns.to_vec(),
                rows: rows.to_vec(),
                rows_affected,
            };
            Ok(serde_json::to_string(&result)?)
        }
        OutputFormat::JsonPretty => {
            let result = QueryResult {
                columns: columns.to_vec(),
                rows: rows.to_vec(),
                rows_affected,
            };
            Ok(serde_json::to_string_pretty(&result)?)
        }
        OutputFormat::Csv => {
            let mut output = String::new();
            output.push_str(&columns.join(","));
            output.push('\n');
            for row in rows {
                output.push_str(
                    &row.iter()
                        .map(|v| {
                            if v.contains(',') || v.contains('"') {
                                format!("\"{}\"", v.replace('"', "\"\""))
                            } else {
                                v.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(","),
                );
                output.push('\n');
            }
            Ok(output)
        }
    }
}

#[derive(Tabled)]
struct TableRow {
    name: String,
    columns: usize,
    #[tabled(rename = "Primary Key")]
    primary_key: String,
    #[tabled(rename = "Foreign Keys")]
    foreign_keys: usize,
}

pub fn format_tables(tables: &[&TableMetadata]) -> Result<String> {
    let rows: Vec<TableRow> = tables
        .iter()
        .map(|t| TableRow {
            name: t.name.clone(),
            columns: t.columns.len(),
            primary_key: t.primary_key.join(", "),
            foreign_keys: t.foreign_keys.len(),
        })
        .collect();

    Ok(Table::new(rows).to_string())
}

#[derive(Tabled)]
struct ColumnRow {
    name: String,
    #[tabled(rename = "Type")]
    data_type: String,
    nullable: String,
    #[tabled(rename = "Default")]
    default_value: String,
    #[tabled(rename = "PK")]
    is_primary_key: String,
    #[tabled(rename = "FK")]
    is_foreign_key: String,
}

pub fn format_columns(columns: &[ColumnMetadata]) -> Result<String> {
    let rows: Vec<ColumnRow> = columns
        .iter()
        .map(|c| ColumnRow {
            name: c.name.clone(),
            data_type: c.data_type.clone(),
            nullable: if c.nullable { "YES" } else { "NO" }.to_string(),
            default_value: c.default_value.clone().unwrap_or_default(),
            is_primary_key: if c.is_primary_key { "✓" } else { "" }.to_string(),
            is_foreign_key: if c.is_foreign_key { "✓" } else { "" }.to_string(),
        })
        .collect();

    Ok(Table::new(rows).to_string())
}

#[derive(Tabled)]
struct RelationshipRow {
    constraint: String,
    #[tabled(rename = "From")]
    from: String,
    #[tabled(rename = "To")]
    to: String,
}

pub fn format_relationships(relationships: &[ForeignKeyRelationship]) -> Result<String> {
    let rows: Vec<RelationshipRow> = relationships
        .iter()
        .map(|r| RelationshipRow {
            constraint: r.constraint_name.clone(),
            from: format!("{}.{}", r.source_table, r.source_column),
            to: format!("{}.{}", r.target_table, r.target_column),
        })
        .collect();

    Ok(Table::new(rows).to_string())
}

#[derive(Serialize)]
pub struct SchemaTablesJson {
    pub tables: Vec<TableJson>,
}

#[derive(Serialize)]
pub struct TableJson {
    pub name: String,
    pub columns: usize,
    pub primary_key: Vec<String>,
    pub foreign_keys: usize,
}

#[derive(Serialize)]
pub struct TableDetailsJson {
    pub name: String,
    pub columns: Vec<ColumnJson>,
}

#[derive(Serialize)]
pub struct ColumnJson {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
}

pub fn format_tables_json(tables: &[&TableMetadata], pretty: bool) -> Result<String> {
    let json_tables: Vec<TableJson> = tables
        .iter()
        .map(|t| TableJson {
            name: t.name.clone(),
            columns: t.columns.len(),
            primary_key: t.primary_key.clone(),
            foreign_keys: t.foreign_keys.len(),
        })
        .collect();

    let result = SchemaTablesJson {
        tables: json_tables,
    };

    if pretty {
        Ok(serde_json::to_string_pretty(&result)?)
    } else {
        Ok(serde_json::to_string(&result)?)
    }
}

pub fn format_table_details_json(table: &TableMetadata, pretty: bool) -> Result<String> {
    let json_columns: Vec<ColumnJson> = table
        .columns
        .iter()
        .map(|c| ColumnJson {
            name: c.name.clone(),
            data_type: c.data_type.clone(),
            nullable: c.nullable,
            default_value: c.default_value.clone(),
            is_primary_key: c.is_primary_key,
            is_foreign_key: c.is_foreign_key,
        })
        .collect();

    let result = TableDetailsJson {
        name: table.name.clone(),
        columns: json_columns,
    };

    if pretty {
        Ok(serde_json::to_string_pretty(&result)?)
    } else {
        Ok(serde_json::to_string(&result)?)
    }
}

#[derive(Serialize)]
pub struct SavedQueriesJson {
    pub queries: Vec<SavedQueryJson>,
}

#[derive(Serialize)]
pub struct SavedQueryJson {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub params: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql: Option<String>,
}

pub fn format_saved_queries_json(
    queries: &HashMap<String, SavedQuery>,
    verbose: bool,
    pretty: bool,
) -> Result<String> {
    let mut json_queries: Vec<SavedQueryJson> = queries
        .iter()
        .map(|(name, query)| SavedQueryJson {
            name: name.clone(),
            description: query.description.clone(),
            params: query.params.clone(),
            sql: if verbose {
                Some(query.sql.clone())
            } else {
                None
            },
        })
        .collect();

    json_queries.sort_by(|a, b| a.name.cmp(&b.name));

    let result = SavedQueriesJson {
        queries: json_queries,
    };

    if pretty {
        Ok(serde_json::to_string_pretty(&result)?)
    } else {
        Ok(serde_json::to_string(&result)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str_table() {
        assert!(matches!(
            OutputFormat::from_str("table").unwrap(),
            OutputFormat::Table
        ));
        assert!(matches!(
            OutputFormat::from_str("TABLE").unwrap(),
            OutputFormat::Table
        ));
    }

    #[test]
    fn test_format_from_str_json() {
        assert!(matches!(
            OutputFormat::from_str("json").unwrap(),
            OutputFormat::Json
        ));
        assert!(matches!(
            OutputFormat::from_str("JSON").unwrap(),
            OutputFormat::Json
        ));
    }

    #[test]
    fn test_format_from_str_json_pretty() {
        assert!(matches!(
            OutputFormat::from_str("json-pretty").unwrap(),
            OutputFormat::JsonPretty
        ));
        assert!(matches!(
            OutputFormat::from_str("pretty").unwrap(),
            OutputFormat::JsonPretty
        ));
    }

    #[test]
    fn test_format_from_str_csv() {
        assert!(matches!(
            OutputFormat::from_str("csv").unwrap(),
            OutputFormat::Csv
        ));
    }

    #[test]
    fn test_format_from_str_invalid() {
        let result = OutputFormat::from_str("invalid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown format: invalid"));
    }
}
