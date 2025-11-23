use anyhow::{Context, Result};
use clap::Parser;

use rds_cli::cache::SchemaCache;
use rds_cli::cli::{Cli, Command, ConfigAction, SavedAction, SchemaAction, SecretAction};
use rds_cli::config::{ApplicationConfig, DatabaseProfile};
use rds_cli::crypto::Crypto;
use rds_cli::db;
use rds_cli::format;
use rds_cli::query_manager::QueryManager;
use rds_cli::secret::SecretManager;
use rds_cli::validator::QueryValidator;

struct CliContext {
    config: ApplicationConfig,
    profile_name: String,
}

impl CliContext {
    fn load(cli: &Cli) -> Result<Self> {
        let config = ApplicationConfig::load(cli.profile.clone())?;
        let profile_name = cli
            .profile
            .as_ref()
            .unwrap_or(&config.defaults.default_profile)
            .clone();

        Ok(Self {
            config,
            profile_name,
        })
    }

    fn get_profile(&self) -> Result<&DatabaseProfile> {
        self.config.get_profile(&self.profile_name)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Config { action } => {
            handle_config(action).await?;
        }
        Command::Schema { action } => {
            handle_schema(action, &cli).await?;
        }
        Command::Query { sql } => {
            handle_query(sql, &cli).await?;
        }
        Command::Refresh => {
            handle_refresh(&cli).await?;
        }
        Command::Run { name, param } => {
            handle_run(name, param, &cli).await?;
        }
        Command::Saved { action, verbose } => {
            handle_saved(action.as_ref(), *verbose, &cli).await?;
        }
        Command::Secret { action } => {
            handle_secret(action).await?;
        }
    }

    Ok(())
}

async fn handle_config(action: &ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Init => {
            let config_path = ApplicationConfig::user_config_path()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;

            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let sample_config = r#"[defaults]
default_profile = "local"
cache_ttl_hours = 24
output_format = "table"

[profiles.local]
type = "postgresql"
host = "localhost"
port = 5432
user = "myuser"
database = "mydb"
schema = "public"

[profiles.local.safety]
default_limit = 1000
max_limit = 10000
timeout_seconds = 10
allowed_operations = ["SELECT", "EXPLAIN", "SHOW"]
"#;

            std::fs::write(&config_path, sample_config)?;
            println!("✓ Configuration initialized at: {}", config_path.display());
            println!("\nNext steps:");
            println!("  1. Edit config:        rds-cli config edit");
            println!("  2. Set password:       export DB_PASSWORD_LOCAL=\"your-password\"");
            println!("  3. Refresh schema:     rds-cli refresh");
        }
        ConfigAction::Show => {
            let config = ApplicationConfig::load(None)?;
            println!("Configuration:");
            println!("  Default profile: {}", config.defaults.default_profile);
            println!("  Cache TTL: {} hours", config.defaults.cache_ttl_hours);
            println!("\nProfiles:");
            for (name, profile) in &config.profiles {
                println!(
                    "  {}: {}://{}:{}/{}",
                    name, profile.db_type, profile.host, profile.port, profile.database
                );
            }
        }
        ConfigAction::Path => {
            if let Some(path) = ApplicationConfig::user_config_path() {
                println!("{}", path.display());
            }
        }
        ConfigAction::Edit => {
            let config_path = ApplicationConfig::user_config_path()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            std::process::Command::new(editor)
                .arg(&config_path)
                .status()?;
        }
    }

    Ok(())
}

async fn handle_schema(action: &SchemaAction, cli: &Cli) -> Result<()> {
    let ctx = CliContext::load(cli)?;
    let cache = SchemaCache::load(&ctx.profile_name)?;

    match action {
        SchemaAction::Find { pattern } => {
            let tables = cache.find_tables(pattern);
            if tables.is_empty() {
                println!("No tables found matching '{}'", pattern);
            } else {
                let output_format = cli
                    .format
                    .as_deref()
                    .and_then(|f| f.parse().ok())
                    .unwrap_or(format::OutputFormat::Table);

                let output = match output_format {
                    format::OutputFormat::Json => format::format_tables_json(&tables, false)?,
                    format::OutputFormat::JsonPretty => format::format_tables_json(&tables, true)?,
                    _ => format::format_tables(&tables)?,
                };
                println!("{}", output);
            }
        }
        SchemaAction::Show { table } => {
            let table_meta = cache.get_table_or_error(table)?;

            let output_format = cli
                .format
                .as_deref()
                .and_then(|f| f.parse().ok())
                .unwrap_or(format::OutputFormat::Table);

            let output = match output_format {
                format::OutputFormat::Json => format::format_table_details_json(table_meta, false)?,
                format::OutputFormat::JsonPretty => {
                    format::format_table_details_json(table_meta, true)?
                }
                _ => {
                    let mut result = format!("Table: {}\n\n", table);
                    result.push_str(&format::format_columns(&table_meta.columns)?);
                    result
                }
            };
            println!("{}", output);
        }
        SchemaAction::Relationships { table, summary } => {
            let table_meta = cache.get_table_or_error(table)?;

            if *summary {
                println!("Relationships for table '{}':", table);
                println!(
                    "  Outbound (Foreign Keys): {}",
                    table_meta.foreign_keys.len()
                );
                println!(
                    "  Inbound (Referenced By): {}",
                    table_meta.referenced_by.len()
                );
            } else {
                println!("Foreign Keys (Outbound):\n");
                if !table_meta.foreign_keys.is_empty() {
                    println!(
                        "{}",
                        format::format_relationships(&table_meta.foreign_keys)?
                    );
                } else {
                    println!("  None");
                }

                println!("\nReferenced By (Inbound):\n");
                if !table_meta.referenced_by.is_empty() {
                    println!(
                        "{}",
                        format::format_relationships(&table_meta.referenced_by)?
                    );
                } else {
                    println!("  None");
                }
            }
        }
    }

    Ok(())
}

async fn handle_query(sql: &str, cli: &Cli) -> Result<()> {
    let ctx = CliContext::load(cli)?;
    let profile = ctx.get_profile()?;

    let validator = QueryValidator::new(profile.safety.clone(), &profile.db_type);
    let validated_sql = validator.validate(sql).context("Query validation failed")?;

    if cli.verbose {
        println!("Original SQL: {}", sql);
        println!("Validated SQL: {}", validated_sql);
    }

    let mut database = db::create_database(&profile.db_type)?;
    database.connect(profile).await?;

    let result = database
        .execute_query(&validated_sql, profile.safety.timeout_seconds)
        .await?;

    let output_format = if let Some(fmt) = &cli.format {
        fmt.parse()?
    } else {
        format::OutputFormat::Table
    };

    let output = format::format_query_result(
        &result.columns,
        &result.rows,
        result.rows_affected,
        output_format,
    )?;

    println!("{}", output);

    Ok(())
}

async fn handle_refresh(cli: &Cli) -> Result<()> {
    let ctx = CliContext::load(cli)?;
    let profile = ctx.get_profile()?;

    println!(
        "Refreshing schema cache for profile '{}'...",
        ctx.profile_name
    );

    let mut database = db::create_database(&profile.db_type)?;
    database.connect(profile).await?;

    let schema = database.extract_schema(profile).await?;

    println!("  Tables: {}", schema.tables.len());
    println!("  Cached at: {}", schema.cached_at);

    schema.save(&ctx.profile_name)?;

    println!("✓ Schema cache refreshed successfully");

    Ok(())
}

async fn handle_run(name: &str, params: &[String], cli: &Cli) -> Result<()> {
    let ctx = CliContext::load(cli)?;
    let query_template = ctx.config.get_saved_query(name)?;

    let mut param_map = std::collections::HashMap::new();
    for p in params {
        let parts: Vec<&str> = p.splitn(2, '=').collect();
        if parts.len() == 2 {
            param_map.insert(parts[0].to_string(), parts[1].to_string());
        } else {
            anyhow::bail!("Invalid parameter format: '{}'. Use key=value", p);
        }
    }

    for required in &query_template.params {
        if !param_map.contains_key(required) {
            anyhow::bail!("Missing required parameter: {}", required);
        }
    }

    let mut sql = query_template.sql.clone();
    for (key, value) in param_map {
        sql = sql.replace(&format!(":{}", key), &value);
    }

    handle_query(&sql, cli).await
}

async fn handle_saved(action: Option<&SavedAction>, verbose: bool, cli: &Cli) -> Result<()> {
    if let Some(action) = action {
        let manager = QueryManager::new()?;

        match action {
            SavedAction::Save {
                name,
                sql,
                description,
            } => {
                manager.save_query(name, sql, description.as_deref())?;
                println!("✓ Query '{}' saved successfully", name);

                let params = QueryManager::extract_params(sql);
                if !params.is_empty() {
                    println!("  Detected parameters: {}", params.join(", "));
                }
            }
            SavedAction::Delete { name } => {
                manager.delete_query(name)?;
                println!("✓ Query '{}' deleted successfully", name);
            }
            SavedAction::Show { name } => {
                let details = manager.show_query(name)?;
                println!("Query: {}", details.name);
                if let Some(desc) = &details.description {
                    println!("Description: {}", desc);
                }
                if !details.params.is_empty() {
                    println!("Parameters: {}", details.params.join(", "));
                }
                println!("\nSQL:\n{}", details.sql);
            }
        }

        return Ok(());
    }

    let ctx = CliContext::load(cli)?;

    if ctx.config.saved_queries.is_empty() {
        println!("No saved queries found.");
        return Ok(());
    }

    let output_format = cli
        .format
        .as_deref()
        .and_then(|f| f.parse().ok())
        .unwrap_or(format::OutputFormat::Table);

    let output = match output_format {
        format::OutputFormat::Json => {
            format::format_saved_queries_json(&ctx.config.saved_queries, verbose, false)?
        }
        format::OutputFormat::JsonPretty => {
            format::format_saved_queries_json(&ctx.config.saved_queries, verbose, true)?
        }
        _ => {
            let mut result = String::from("Saved Queries:\n\n");
            for (name, query) in &ctx.config.saved_queries {
                result.push_str(&format!("  {}\n", name));
                if let Some(desc) = &query.description {
                    result.push_str(&format!("    {}\n", desc));
                }
                if !query.params.is_empty() {
                    result.push_str(&format!("    Parameters: {}\n", query.params.join(", ")));
                }
                if verbose {
                    let preview = query.sql.lines().next().unwrap_or("");
                    result.push_str(&format!("    SQL: {}...\n", preview));
                }
                result.push('\n');
            }
            result
        }
    };

    println!("{}", output);

    Ok(())
}
async fn handle_secret(action: &SecretAction) -> Result<()> {
    let secret_mgr = SecretManager::new()?;
    let master_key = secret_mgr.get_or_create_master_key()?;
    let crypto = Crypto::new(&master_key);

    match action {
        SecretAction::Set { profile, password_stdin } => {
            let config_path = ApplicationConfig::project_config_path()
                .context("No project config found (.rds-cli.toml)")?;

            let password = if *password_stdin {
                use std::io::{self, Read};
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                buffer.trim().to_string()
            } else {
                rpassword::prompt_password(format!("Password for profile '{}': ", profile))
                    .context("Failed to read password")?
            };

            let encrypted = crypto.encrypt(&password)?;

            let mut config = if config_path.exists() {
                ApplicationConfig::from_file(&config_path)?
            } else {
                ApplicationConfig::default()
            };

            if let Some(profile_config) = config.profiles.get_mut(profile) {
                profile_config.password = encrypted;
            } else {
                anyhow::bail!("Profile '{}' not found in .rds-cli.toml", profile);
            }

            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;

            println!("✓ Encrypted password set for profile '{}'", profile);
        }
        SecretAction::Get { profile } => {
            let config = ApplicationConfig::load(None)?;
            let profile_config = config.get_profile(profile)?;

            if profile_config.password.starts_with("enc:") {
                let decrypted = crypto.decrypt(&profile_config.password)?;
                println!("{}", decrypted);
            } else if !profile_config.password.is_empty() {
                println!("{}", profile_config.password);
            } else {
                println!("(empty)");
            }
        }
        SecretAction::Remove { profile } => {
            let config_path = ApplicationConfig::project_config_path()
                .context("No project config found (.rds-cli.toml)")?;

            let mut config = ApplicationConfig::from_file(&config_path)?;

            if let Some(profile_config) = config.profiles.get_mut(profile) {
                profile_config.password = String::new();
            } else {
                anyhow::bail!("Profile '{}' not found", profile);
            }

            let content = toml::to_string_pretty(&config)?;
            std::fs::write(&config_path, content)?;

            println!("✓ Password removed for profile '{}'", profile);
        }
        SecretAction::Reset => {
            secret_mgr.reset_master_key()?;
            println!("✓ Master key reset successfully");
            println!("⚠️  All encrypted passwords are now unrecoverable");
            println!("   Use 'rds-cli secret set <profile>' to re-encrypt passwords");
        }
    }

    Ok(())
}
