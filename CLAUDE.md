# RDS CLI - AI Agent Developer Guide

This is a Rust CLI tool for safe PostgreSQL/MySQL database operations. Focus: schema caching, query validation, and team-shared named queries.

---

## Architecture Overview

### Database Trait Abstraction

```rust
#[async_trait]
pub trait Database: Send + Sync {
    async fn connect(&mut self, profile: &DatabaseProfile) -> Result<()>;
    async fn extract_schema(&self, profile: &DatabaseProfile) -> Result<SchemaCache>;
    async fn execute_query(&self, sql: &str, timeout_secs: u64) -> Result<QueryResult>;
    fn db_type(&self) -> &str;
}
```

**Key Decision**: Factory pattern in `src/db/mod.rs` creates `Box<dyn Database>`. Add new databases by implementing these 4 methods.

**Implementations**:
- `PostgresDatabase` (`src/db/postgres.rs`): tokio-postgres, extracts from `information_schema`
- `MySqlDatabase` (`src/db/mysql.rs`): mysql_async Pool, uses `information_schema.COLUMNS`

---

### 5-Level Configuration Hierarchy

Priority: **CLI args > ENV vars > Project config > User config > Defaults**

```rust
pub fn load(cli_profile: Option<String>) -> Result<Self> {
    let mut config = Self::default();
    if user_config.exists() { config = config.merge(user_config); }
    if project_config.exists() { config = config.merge(project_config); }
    config.load_env_vars()?;  // DB_PASSWORD_<PROFILE> injection
    if cli_profile.is_some() { config.defaults.default_profile = cli_profile; }
}
```

**Key Decision**: `merge()` is non-destructive. Allows team to share `.rds-cli.toml` (project) while users override with `~/.config/rds-cli/application.toml`.

**Password Handling**: Environment variables only (`DB_PASSWORD_<PROFILE>`). Never stored in config files.

---

### Schema Caching

```rust
pub struct SchemaCache {
    pub cached_at: DateTime<Utc>,
    pub profile_name: String,
    pub database_type: String,
    pub tables: HashMap<String, TableMetadata>,
}
```

**Location**: `~/.config/rds-cli/cache/<profile>/schema.json`

**Key Decision**: JSON serialization for <5ms lookups. Avoids database roundtrips on every schema operation.

**Fuzzy Search**: Levenshtein distance (via `strsim`), max distance 3, shows top 3 suggestions.

**Methods**:
- `get_table_or_error()`: Returns table or prints fuzzy suggestions
- `suggest_tables()`: Returns `Vec<(String, usize)>` sorted by distance

---

### Query Validation & Safety

```rust
pub fn validate(&self, sql: &str) -> Result<String> {
    let statements = Parser::parse_sql(&*self.dialect, sql)?;

    // Only Statement::Query(_) allowed
    for statement in &statements {
        match statement {
            Statement::Query(_) => {}
            _ => anyhow::bail!("Only SELECT queries allowed"),
        }
    }

    // Auto LIMIT injection if missing
    if !self.has_limit(sql) {
        return Ok(format!("{} LIMIT {}", sql.trim_end_matches(';'), self.policy.default_limit));
    }
}
```

**Key Decision**: `sqlparser` with database-specific dialects (PostgreSQL/MySQL). String-based LIMIT detection (not AST) for simplicity.

**Enforcement**: Only `Statement::Query` allowed by default. Production profiles set `allowed_operations = ["SELECT"]`.

---

### Named Queries with Parameter Detection

**Storage**: `.rds-cli.toml` (project-level, Git-versioned)

**Key Decision**: `toml_edit` crate preserves formatting/comments. Enables clean Git diffs.

```rust
pub fn extract_params(sql: &str) -> Vec<String> {
    let re = Regex::new(r":(\w+)").unwrap();
    let mut params: Vec<String> = re.captures_iter(sql)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect();
    params.sort();
    params.dedup();
    params
}
```

**Parameter Syntax**: `:param_name` (named parameters). Auto-detected via regex.

---

## Implementation Patterns

### CliContext Pattern

**Location**: `src/main.rs`

```rust
struct CliContext {
    config: ApplicationConfig,
    profile_name: String,
}

impl CliContext {
    fn load(cli: &Cli) -> Result<Self>  // Single config loading point
    fn get_profile(&self) -> Result<&DatabaseProfile>
}
```

**Key Decision**: Eliminates duplicate config loading. All handlers use `CliContext::load(cli)`.

**Handler Pattern**: `async fn handle_*(args, cli: &Cli) -> Result<()>`

---

### Error Handling

**Strategy**: `anyhow::Result` everywhere. `anyhow::Context` for error chains.

```rust
Self::from_file(&path).context("Failed to read config")?
```

**Key Decision**: No custom error types. CLI tools don't need enum variants.

---

## Development Tasks

### Add New Database

1. Create `src/db/newdb.rs`:
```rust
pub struct NewDatabase { connection: Option<Connection> }

#[async_trait]
impl Database for NewDatabase {
    async fn connect(&mut self, profile: &DatabaseProfile) -> Result<()> { /* */ }
    async fn extract_schema(&self, profile: &DatabaseProfile) -> Result<SchemaCache> { /* */ }
    async fn execute_query(&self, sql: &str, timeout_secs: u64) -> Result<QueryResult> { /* */ }
    fn db_type(&self) -> &str { "newdb" }
}
```

2. Register in factory (`src/db/mod.rs`):
```rust
pub fn create_database(db_type: &str) -> Result<Box<dyn Database>> {
    match db_type {
        "postgresql" => Ok(Box::new(postgres::PostgresDatabase::new())),
        "mysql" => Ok(Box::new(mysql::MySqlDatabase::new())),
        "newdb" => Ok(Box::new(newdb::NewDatabase::new())),
        _ => anyhow::bail!("Unsupported database type: {}", db_type),
    }
}
```

---

### Add New Command

1. `src/cli.rs`:
```rust
#[derive(Subcommand)]
pub enum Command {
    NewCommand {
        #[arg(long)] param: String
    },
}
```

2. `src/main.rs`:
```rust
Command::NewCommand { param } => handle_new_command(&param, &cli).await?,

async fn handle_new_command(param: &str, cli: &Cli) -> Result<()> {
    let ctx = CliContext::load(cli)?;
    let profile = ctx.get_profile()?;
    // Use SchemaCache::load(&ctx.profile_name)?
    // Use format::OutputFormat::from_str()?
}
```

---

### Modify Configuration

1. Add field to `ApplicationConfig` (`src/config.rs`)
2. Update `merge()` logic with non-destructive override
3. Update init template in `src/main.rs` if config file default needed

---

## Common Issues & Fixes

### Cache Not Found

**Symptom**: `Cache not found for profile 'xxx'`

**Cause**: `rds-cli refresh` never run

**Fix**: Run `rds-cli refresh` to generate cache at `~/.config/rds-cli/cache/<profile>/schema.json`

---

### Table Not Found Despite Existing

**Cause**: Stale cache or case-sensitivity

**Fix**:
```bash
rds-cli schema find xxx  # Check cached tables
rds-cli refresh          # Regenerate cache
```

**Debug**: Check `SchemaCache::get_table_or_error()` shows fuzzy suggestions

---

### Query Validation Failed

**Symptom**: `Only SELECT queries allowed`

**Cause**: Safety policy blocking non-SELECT

**Fix**: Modify profile's `allowed_operations`:
```toml
[profiles.dev.safety]
allowed_operations = ["SELECT", "INSERT", "UPDATE", "DELETE"]
```

**Production**: Keep `["SELECT"]` only

---

### Connection Failed

**Checklist**:
1. Database running: `psql` or `mysql` test
2. Config correct: `rds-cli config show`
3. Password set: `echo $DB_PASSWORD_<PROFILE>`
4. Network/firewall

**Debug**: `rds-cli --verbose query "SELECT 1"`

---

### CliContext Pattern Not Used

**Symptom**: Duplicate `ApplicationConfig::load()` calls

**Fix**: Use `CliContext::load(cli)?` in all handlers

---

## Key Files

- `src/main.rs`: CLI handlers, CliContext pattern
- `src/config.rs`: 5-level hierarchy, merge logic, env var loading
- `src/cache.rs`: Schema caching, fuzzy search
- `src/validator.rs`: SQL parsing, LIMIT injection
- `src/query_manager.rs`: Named queries, parameter extraction, toml_edit
- `src/db/mod.rs`: Database trait, factory
- `src/db/postgres.rs`: PostgreSQL implementation
- `src/db/mysql.rs`: MySQL implementation
- `src/format.rs`: Output formatting (table, json, csv)

---

## Performance

**Binary**: 6.7MB (LTO + strip)

**Bottlenecks**:
- Schema cache: <5ms (JSON deserialization)
- SQL validation: <1ms (sqlparser)
- Query execution: Database I/O dependent

**Config**: `Cargo.toml` release profile uses `lto = true`, `strip = true`, `opt-level = 3`

---

For user documentation, see [README.md](README.md).
