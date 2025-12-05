use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use toml_edit::{DocumentMut, Item, Table};

static PARAM_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":(\w+)").expect("Invalid regex pattern"));

pub struct QueryManager {
    config_path: PathBuf,
}

impl QueryManager {
    pub fn new() -> Result<Self> {
        let config_path = crate::config::ApplicationConfig::project_config_path()
            .context("Not in a project directory")?;

        Ok(Self { config_path })
    }

    pub fn save_query(&self, name: &str, sql: &str, description: Option<&str>) -> Result<()> {
        let params = Self::extract_params(sql);

        let mut doc = if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            content.parse::<DocumentMut>()?
        } else {
            DocumentMut::new()
        };

        if !doc.contains_key("saved_queries") {
            doc["saved_queries"] = Item::Table(Table::new());
        }

        let saved_queries = doc["saved_queries"]
            .as_table_mut()
            .context("saved_queries is not a table")?;

        let mut query_table = Table::new();
        query_table["sql"] = toml_edit::value(sql);

        if let Some(desc) = description {
            query_table["description"] = toml_edit::value(desc);
        }

        if !params.is_empty() {
            let mut params_array = toml_edit::Array::new();
            for param in &params {
                params_array.push(param.as_str());
            }
            query_table["params"] = toml_edit::value(params_array);
        }

        saved_queries[name] = Item::Table(query_table);

        fs::write(&self.config_path, doc.to_string())?;

        Ok(())
    }

    pub fn delete_query(&self, name: &str) -> Result<()> {
        if !self.config_path.exists() {
            anyhow::bail!("Config file not found: {}", self.config_path.display());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        if let Some(saved_queries) = doc.get_mut("saved_queries").and_then(|v| v.as_table_mut()) {
            if saved_queries.remove(name).is_none() {
                anyhow::bail!("Query '{}' not found", name);
            }
        } else {
            anyhow::bail!("No saved queries found");
        }

        fs::write(&self.config_path, doc.to_string())?;

        Ok(())
    }

    pub fn show_query(&self, name: &str) -> Result<QueryDetails> {
        if !self.config_path.exists() {
            anyhow::bail!("Config file not found: {}", self.config_path.display());
        }

        let content = fs::read_to_string(&self.config_path)?;
        let doc = content.parse::<DocumentMut>()?;

        let saved_queries = doc
            .get("saved_queries")
            .and_then(|v| v.as_table())
            .context("No saved queries found")?;

        let query_table = saved_queries
            .get(name)
            .and_then(|v| v.as_table())
            .context(format!("Query '{}' not found", name))?;

        let sql = query_table
            .get("sql")
            .and_then(|v| v.as_str())
            .context("Query has no SQL")?
            .to_string();

        let description = query_table
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let params = query_table
            .get("params")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(QueryDetails {
            name: name.to_string(),
            sql,
            description,
            params,
        })
    }

    pub fn extract_params(sql: &str) -> Vec<String> {
        let mut params: Vec<String> = PARAM_REGEX
            .captures_iter(sql)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        params.sort();
        params.dedup();
        params
    }
}

pub struct QueryDetails {
    pub name: String,
    pub sql: String,
    pub description: Option<String>,
    pub params: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_params_single() {
        let sql = "SELECT * FROM users WHERE email = :email";
        let params = QueryManager::extract_params(sql);
        assert_eq!(params, vec!["email"]);
    }

    #[test]
    fn test_extract_params_multiple() {
        let sql = "SELECT * FROM orders WHERE user_id = :user_id AND status = :status";
        let params = QueryManager::extract_params(sql);
        assert_eq!(params, vec!["status", "user_id"]); // sorted
    }

    #[test]
    fn test_extract_params_duplicates() {
        let sql = "SELECT * FROM logs WHERE :date >= start AND :date <= end";
        let params = QueryManager::extract_params(sql);
        assert_eq!(params, vec!["date"]); // deduplicated
    }

    #[test]
    fn test_extract_params_none() {
        let sql = "SELECT * FROM users";
        let params = QueryManager::extract_params(sql);
        assert_eq!(params, Vec::<String>::new());
    }

    #[test]
    fn test_extract_params_complex() {
        let sql = "SELECT * FROM orders WHERE user_id = :user_id AND status = :status LIMIT :limit";
        let params = QueryManager::extract_params(sql);
        assert_eq!(params, vec!["limit", "status", "user_id"]); // sorted
    }
}
