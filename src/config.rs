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

    pub fn user_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("rds-cli");
            p.push("application.toml");
            p
        })
    }

    pub fn project_config_path() -> Option<PathBuf> {
        std::env::current_dir().ok().map(|mut p| {
            p.push(".rds-cli.toml");
            p
        })
    }

    fn from_file(path: &PathBuf) -> Result<Self> {
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
        for (profile_name, profile) in &mut self.profiles {
            let env_var = format!("DB_PASSWORD_{}", profile_name.to_uppercase());
            if let Ok(password) = std::env::var(&env_var) {
                profile.password = password;
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
