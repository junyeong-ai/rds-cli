use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rds-cli", version, about = "Universal RDS CLI Tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[arg(long, short, global = true)]
    pub profile: Option<String>,

    #[arg(long, short, global = true)]
    pub format: Option<String>,

    #[arg(long, short, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Command {
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    Schema {
        #[command(subcommand)]
        action: SchemaAction,
    },
    Query {
        sql: String,
    },
    Refresh,
    Run {
        name: String,
        #[arg(short, long)]
        param: Vec<String>,
    },
    Saved {
        #[command(subcommand)]
        action: Option<SavedAction>,
        #[arg(long)]
        verbose: bool,
    },
    Secret {
        #[command(subcommand)]
        action: SecretAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Init,
    Show,
    Path,
    Edit,
}

#[derive(Subcommand)]
pub enum SchemaAction {
    Find {
        pattern: String,
    },
    Show {
        table: String,
    },
    Relationships {
        table: String,
        #[arg(long)]
        summary: bool,
    },
}

#[derive(Subcommand)]
pub enum SavedAction {
    Save {
        name: String,
        sql: String,
        #[arg(short, long)]
        description: Option<String>,
    },
    Delete {
        name: String,
    },
    Show {
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SecretAction {
    Set {
        profile: String,
        #[arg(long)]
        password_stdin: bool,
    },
    Get {
        profile: String,
    },
    Remove {
        profile: String,
    },
}
