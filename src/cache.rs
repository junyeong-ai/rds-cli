use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaCache {
    pub cached_at: DateTime<Utc>,
    pub profile_name: String,
    pub database_type: String,
    pub tables: HashMap<String, TableMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMetadata {
    pub name: String,
    pub columns: Vec<ColumnMetadata>,
    #[serde(default)]
    pub primary_key: Vec<String>,
    #[serde(default)]
    pub foreign_keys: Vec<ForeignKeyRelationship>,
    #[serde(default)]
    pub referenced_by: Vec<ForeignKeyRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMetadata {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub is_primary_key: bool,
    #[serde(default)]
    pub is_foreign_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyRelationship {
    pub constraint_name: String,
    pub source_table: String,
    pub source_column: String,
    pub target_table: String,
    pub target_column: String,
}

impl SchemaCache {
    pub fn cache_path(profile: &str) -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;

        path.push("rds-cli");
        path.push("cache");
        path.push(profile);

        fs::create_dir_all(&path)?;

        path.push("schema.json");
        Ok(path)
    }

    pub fn save(&self, profile: &str) -> Result<()> {
        let path = Self::cache_path(profile)?;
        let file = File::create(&path)
            .with_context(|| format!("Failed to create cache file: {}", path.display()))?;

        serde_json::to_writer_pretty(file, self)
            .with_context(|| format!("Failed to write cache: {}", path.display()))?;

        Ok(())
    }

    pub fn load(profile: &str) -> Result<Self> {
        let path = Self::cache_path(profile)?;

        if !path.exists() {
            anyhow::bail!(
                "Cache not found for profile '{}'\nRun: rds-cli refresh",
                profile
            );
        }

        let file = File::open(&path)
            .with_context(|| format!("Failed to open cache: {}", path.display()))?;

        serde_json::from_reader(file)
            .with_context(|| format!("Failed to parse cache: {}", path.display()))
    }

    pub fn find_tables(&self, pattern: &str) -> Vec<&TableMetadata> {
        self.tables
            .values()
            .filter(|table| table.name.to_lowercase().contains(&pattern.to_lowercase()))
            .collect()
    }

    pub fn get_table(&self, name: &str) -> Option<&TableMetadata> {
        self.tables.get(name)
    }

    pub fn suggest_tables(&self, name: &str) -> Vec<(String, usize)> {
        let mut suggestions: Vec<(String, usize)> = self
            .tables
            .keys()
            .map(|table_name| {
                let distance = strsim::levenshtein(name, table_name);
                (table_name.clone(), distance)
            })
            .filter(|(_, dist)| *dist <= 3)
            .collect();

        suggestions.sort_by_key(|(_, dist)| *dist);
        suggestions.truncate(3);
        suggestions
    }

    pub fn get_table_or_error(&self, name: &str) -> anyhow::Result<&TableMetadata> {
        if let Some(table) = self.get_table(name) {
            return Ok(table);
        }

        eprintln!("❌ Table '{}' not found\n", name);

        let suggestions = self.suggest_tables(name);
        if !suggestions.is_empty() {
            eprintln!("Did you mean one of these?");
            for (suggestion, _) in suggestions {
                if let Some(meta) = self.get_table(&suggestion) {
                    eprintln!("  - {} ({} columns)", suggestion, meta.columns.len());
                }
            }
            eprintln!("\nRun: rds-cli schema find {}", &name[..name.len().min(3)]);
        }

        anyhow::bail!("Table not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_cache() -> SchemaCache {
        let mut tables = HashMap::new();
        
        tables.insert(
            "users".to_string(),
            TableMetadata {
                name: "users".to_string(),
                columns: vec![],
                primary_key: vec![],
                foreign_keys: vec![],
                referenced_by: vec![],
            },
        );
        
        tables.insert(
            "user_roles".to_string(),
            TableMetadata {
                name: "user_roles".to_string(),
                columns: vec![],
                primary_key: vec![],
                foreign_keys: vec![],
                referenced_by: vec![],
            },
        );
        
        tables.insert(
            "orders".to_string(),
            TableMetadata {
                name: "orders".to_string(),
                columns: vec![],
                primary_key: vec![],
                foreign_keys: vec![],
                referenced_by: vec![],
            },
        );

        SchemaCache {
            cached_at: Utc::now(),
            profile_name: "test".to_string(),
            database_type: "postgresql".to_string(),
            tables,
        }
    }

    #[test]
    fn test_find_tables_exact() {
        let cache = create_test_cache();
        let results = cache.find_tables("users");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "users");
    }

    #[test]
    fn test_find_tables_partial() {
        let cache = create_test_cache();
        let results = cache.find_tables("user");
        assert_eq!(results.len(), 2); // users and user_roles
    }

    #[test]
    fn test_find_tables_case_insensitive() {
        let cache = create_test_cache();
        let results = cache.find_tables("USER");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_find_tables_no_match() {
        let cache = create_test_cache();
        let results = cache.find_tables("nonexistent");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_suggest_tables_exact() {
        let cache = create_test_cache();
        let suggestions = cache.suggest_tables("users");
        assert!(suggestions.len() > 0);
        assert_eq!(suggestions[0].0, "users");
        assert_eq!(suggestions[0].1, 0); // distance 0 (exact match comes first)
    }

    #[test]
    fn test_suggest_tables_typo() {
        let cache = create_test_cache();
        let suggestions = cache.suggest_tables("user");
        assert!(suggestions.len() > 0);
        // "users" should be first (distance 1)
        assert_eq!(suggestions[0].0, "users");
        assert_eq!(suggestions[0].1, 1);
    }

    #[test]
    fn test_suggest_tables_max_distance() {
        let cache = create_test_cache();
        let suggestions = cache.suggest_tables("usr");
        assert!(suggestions.len() > 0);
        // distance should be <= 3
        for (_, dist) in &suggestions {
            assert!(*dist <= 3);
        }
    }

    #[test]
    fn test_suggest_tables_sorted_by_distance() {
        let cache = create_test_cache();
        let suggestions = cache.suggest_tables("user");
        // Should be sorted by distance (ascending)
        for i in 1..suggestions.len() {
            assert!(suggestions[i].1 >= suggestions[i - 1].1);
        }
    }

    #[test]
    fn test_suggest_tables_max_3_results() {
        let cache = create_test_cache();
        let suggestions = cache.suggest_tables("o");
        assert!(suggestions.len() <= 3);
    }
}
