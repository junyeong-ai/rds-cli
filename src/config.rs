use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ApplicationConfig {
    #[serde(default)]
    pub profiles: HashMap<String, DatabaseProfile>,

    #[serde(default)]
    pub defaults: DefaultSettings,

    #[serde(default)]
    pub saved_queries: HashMap<String, SavedQuery>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SavedQuery {
    pub sql: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub params: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefaultSettings {
    #[serde(default = "default_profile")]
    pub default_profile: String,

    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_hours: u32,

    #[serde(default = "default_output_format")]
    pub output_format: String,
}

fn default_profile() -> String {
    "local".to_string()
}

fn default_cache_ttl() -> u32 {
    24
}

fn default_output_format() -> String {
    "table".to_string()
}

impl Default for DefaultSettings {
    fn default() -> Self {
        Self {
            default_profile: default_profile(),
            cache_ttl_hours: default_cache_ttl(),
            output_format: default_output_format(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseProfile {
    #[serde(rename = "type")]
    pub db_type: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    #[serde(default)]
    pub password: String,
    pub database: String,
    #[serde(default)]
    pub schema: Option<String>,
    pub safety: SafetyPolicy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SafetyPolicy {
    pub default_limit: u32,
    pub max_limit: u32,
    pub timeout_seconds: u64,
    pub allowed_operations: Vec<String>,
}

impl ApplicationConfig {
    pub fn load(cli_profile: Option<String>) -> Result<Self> {
        let mut config = Self::default();

        if let Some(path) = Self::user_config_path()
            && path.exists()
        {
            let user_config = Self::from_file(&path)?;
            config = config.merge(user_config);
        }

        if let Some(path) = Self::project_config_path()
            && path.exists()
        {
            let project_config = Self::from_file(&path)?;
            config = config.merge(project_config);
        }

        config.load_env_vars()?;

        if let Some(profile) = cli_profile {
            config.defaults.default_profile = profile;
        }

        Ok(config)
    }

    /// Returns the base config directory: ~/.config/rds-cli
    /// Follows XDG Base Directory standard on all platforms (Unix/macOS)
    pub fn config_base_dir() -> Option<PathBuf> {
        std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|home| PathBuf::from(home).join(".config"))
            })
            .map(|mut p| {
                p.push("rds-cli");
                p
            })
    }

    pub fn user_config_path() -> Option<PathBuf> {
        Self::config_base_dir().map(|mut p| {
            p.push("config.toml");
            p
        })
    }

    pub fn project_config_path() -> Option<PathBuf> {
        std::env::current_dir().ok().map(|mut p| {
            p.push(".rds-cli.toml");
            p
        })
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config: {}", path.display()))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config: {}", path.display()))
    }

    fn merge(mut self, other: Self) -> Self {
        for (name, profile) in other.profiles {
            self.profiles.insert(name, profile);
        }

        for (name, query) in other.saved_queries {
            self.saved_queries.insert(name, query);
        }

        if !other.defaults.default_profile.is_empty() {
            self.defaults.default_profile = other.defaults.default_profile;
        }

        if other.defaults.cache_ttl_hours > 0 {
            self.defaults.cache_ttl_hours = other.defaults.cache_ttl_hours;
        }

        if !other.defaults.output_format.is_empty() {
            self.defaults.output_format = other.defaults.output_format;
        }

        self
    }

    fn load_env_vars(&mut self) -> Result<()> {
        use crate::crypto::Crypto;
        use crate::secret::SecretManager;

        let decrypt = || -> Result<Crypto> {
            let secret_mgr = SecretManager::new()?;
            let master_key = secret_mgr.get_or_create_master_key()?;
            Ok(Crypto::new(&master_key))
        };

        let crypto = decrypt().ok();

        for (profile_name, profile) in &mut self.profiles {
            if profile.password.starts_with("enc:") {
                if let Some(ref c) = crypto
                    && let Ok(decrypted) = c.decrypt(&profile.password)
                {
                    profile.password = decrypted;
                }
            } else {
                let env_var = format!("DB_PASSWORD_{}", profile_name.to_uppercase());
                if let Ok(password) = std::env::var(&env_var) {
                    profile.password = password;
                }
            }
        }
        Ok(())
    }

    pub fn get_profile(&self, name: &str) -> Result<&DatabaseProfile> {
        self.profiles
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", name))
    }

    pub fn get_saved_query(&self, name: &str) -> Result<&SavedQuery> {
        self.saved_queries
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Saved query '{}' not found", name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let defaults = DefaultSettings::default();
        assert_eq!(defaults.default_profile, "local");
        assert_eq!(defaults.cache_ttl_hours, 24);
        assert_eq!(defaults.output_format, "table");
    }

    #[test]
    fn test_merge_profiles() {
        let mut config1 = ApplicationConfig::default();
        config1.profiles.insert(
            "local".to_string(),
            DatabaseProfile {
                db_type: "postgresql".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: "user1".to_string(),
                password: "".to_string(),
                database: "db1".to_string(),
                schema: Some("public".to_string()),
                safety: SafetyPolicy {
                    default_limit: 1000,
                    max_limit: 10000,
                    timeout_seconds: 10,
                    allowed_operations: vec!["SELECT".to_string()],
                },
            },
        );

        let mut config2 = ApplicationConfig::default();
        config2.profiles.insert(
            "production".to_string(),
            DatabaseProfile {
                db_type: "mysql".to_string(),
                host: "prod.example.com".to_string(),
                port: 3306,
                user: "readonly".to_string(),
                password: "".to_string(),
                database: "prod_db".to_string(),
                schema: None,
                safety: SafetyPolicy {
                    default_limit: 100,
                    max_limit: 1000,
                    timeout_seconds: 5,
                    allowed_operations: vec!["SELECT".to_string()],
                },
            },
        );

        let merged = config1.merge(config2);
        assert_eq!(merged.profiles.len(), 2);
        assert!(merged.profiles.contains_key("local"));
        assert!(merged.profiles.contains_key("production"));
    }

    #[test]
    fn test_merge_override_profile() {
        let mut config1 = ApplicationConfig::default();
        config1.profiles.insert(
            "local".to_string(),
            DatabaseProfile {
                db_type: "postgresql".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                user: "user1".to_string(),
                password: "".to_string(),
                database: "db1".to_string(),
                schema: Some("public".to_string()),
                safety: SafetyPolicy {
                    default_limit: 1000,
                    max_limit: 10000,
                    timeout_seconds: 10,
                    allowed_operations: vec!["SELECT".to_string()],
                },
            },
        );

        let mut config2 = ApplicationConfig::default();
        config2.profiles.insert(
            "local".to_string(),
            DatabaseProfile {
                db_type: "mysql".to_string(),
                host: "other.example.com".to_string(),
                port: 3306,
                user: "user2".to_string(),
                password: "".to_string(),
                database: "db2".to_string(),
                schema: None,
                safety: SafetyPolicy {
                    default_limit: 100,
                    max_limit: 1000,
                    timeout_seconds: 5,
                    allowed_operations: vec!["SELECT".to_string()],
                },
            },
        );

        let merged = config1.merge(config2);
        assert_eq!(merged.profiles.len(), 1);
        let local = merged.profiles.get("local").unwrap();
        assert_eq!(local.db_type, "mysql"); // overridden
        assert_eq!(local.host, "other.example.com"); // overridden
    }

    #[test]
    fn test_merge_saved_queries() {
        let mut config1 = ApplicationConfig::default();
        config1.saved_queries.insert(
            "query1".to_string(),
            SavedQuery {
                sql: "SELECT 1".to_string(),
                description: Some("Test 1".to_string()),
                params: vec![],
            },
        );

        let mut config2 = ApplicationConfig::default();
        config2.saved_queries.insert(
            "query2".to_string(),
            SavedQuery {
                sql: "SELECT 2".to_string(),
                description: Some("Test 2".to_string()),
                params: vec![],
            },
        );

        let merged = config1.merge(config2);
        assert_eq!(merged.saved_queries.len(), 2);
        assert!(merged.saved_queries.contains_key("query1"));
        assert!(merged.saved_queries.contains_key("query2"));
    }

    #[test]
    fn test_merge_defaults() {
        let mut config1 = ApplicationConfig::default();
        config1.defaults.default_profile = "local".to_string();

        let mut config2 = ApplicationConfig::default();
        config2.defaults.default_profile = "production".to_string();
        config2.defaults.cache_ttl_hours = 48;

        let merged = config1.merge(config2);
        assert_eq!(merged.defaults.default_profile, "production"); // overridden
        assert_eq!(merged.defaults.cache_ttl_hours, 48); // overridden
    }
}
