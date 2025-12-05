use clap::{Parser, Subcommand};

use crate::format::OutputFormat;

#[derive(Parser)]
#[command(
    name = "rds-cli",
    version,
    about = "Universal RDS CLI Tool",
    long_about = "Safe PostgreSQL/MySQL CLI with schema caching, query validation, and encrypted passwords"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, short, global = true, help = "Database profile to use")]
    pub profile: Option<String>,

    #[arg(long, short, global = true, value_enum, help = "Output format")]
    pub format: Option<OutputFormat>,

    #[arg(long, short, global = true, help = "Enable verbose output")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize and manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Explore database schema (cached)
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
    /// Execute SQL query with safety policies
    Query {
        #[arg(help = "SQL query to execute")]
        sql: String,
    },
    /// Refresh schema cache from database
    Refresh,
    /// Execute a saved query
    Run {
        #[arg(help = "Name of the saved query")]
        name: String,
        #[arg(short = 'a', long = "arg", help = "Parameters in key=value format")]
        args: Vec<String>,
    },
    /// Manage saved queries
    Saved {
        #[command(subcommand)]
        action: SavedAction,
    },
    /// Manage encrypted passwords
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Create a new project configuration file
    Init,
    /// Show current configuration
    Show,
    /// Show configuration file path
    Path {
        #[arg(long, help = "Show global config path instead of project config")]
        global: bool,
    },
    /// Open configuration in editor
    Edit,
}

#[derive(Subcommand)]
pub enum SchemaAction {
    /// Find tables matching a pattern
    Find {
        #[arg(help = "Pattern to search for (case-insensitive substring match)")]
        pattern: String,
    },
    /// Show table details (columns, types, constraints)
    Show {
        #[arg(help = "Table name")]
        table: String,
    },
    /// Show table relationships (foreign keys)
    Relationships {
        #[arg(help = "Table name")]
        table: String,
        #[arg(long, help = "Show summary only")]
        summary: bool,
    },
}

#[derive(Subcommand)]
pub enum SavedAction {
    /// List all saved queries
    #[command(name = "list")]
    List {
        #[arg(long, help = "Show SQL preview")]
        verbose: bool,
    },
    /// Save a new query
    Save {
        #[arg(help = "Query name")]
        name: String,
        #[arg(help = "SQL query")]
        sql: String,
        #[arg(short, long, help = "Query description")]
        description: Option<String>,
    },
    /// Delete a saved query
    Delete {
        #[arg(help = "Query name to delete")]
        name: String,
    },
    /// Show details of a saved query
    Show {
        #[arg(help = "Query name")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SecretAction {
    /// Set encrypted password for a profile
    Set {
        #[arg(help = "Profile name")]
        profile: String,
        #[arg(long, help = "Read password from stdin")]
        password_stdin: bool,
    },
    /// Get decrypted password for a profile
    Get {
        #[arg(help = "Profile name")]
        profile: String,
    },
    /// Remove password for a profile
    Remove {
        #[arg(help = "Profile name")]
        profile: String,
    },
    /// Reset master key (all passwords become unrecoverable)
    Reset,
}
