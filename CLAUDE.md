# RDS CLI - AI Agent Guide

Rust CLI for safe PostgreSQL/MySQL operations. Focus: schema caching, query validation, encrypted passwords.

---

## Architecture

### Database Trait

Factory pattern in `src/db/mod.rs`. Add databases by implementing 4 methods: `connect`, `extract_schema`, `execute_query`, `db_type`.

**Implementations**: PostgreSQL (`postgres.rs`), MySQL (`mysql.rs`)

---

### Configuration Hierarchy

Priority: **CLI args > Encrypted password > ENV vars > Project config > User config > Defaults**

**Key Decisions**:
- `merge()` is non-destructive
- Encrypted passwords auto-decrypt during `load_env_vars()`
- User config: `~/.config/rds-cli/config.toml`
- Project config: `./.rds-cli.toml` (Git-safe with encryption)

---

### Encrypted Passwords

**Master Key**: `~/.config/rds-cli/.master.key` (32-byte random, 0600 permissions)
**Encryption**: ChaCha20-Poly1305, format `enc:base64(nonce || ciphertext || tag)`
**Storage**: `.rds-cli.toml` as `password = "enc:..."`
**Auto-decryption**: `load_env_vars()` decrypts `enc:...` passwords automatically
**Commands**: `secret set/get/remove/reset <profile>`

**Key Decision**: File-based key (not OS Keyring) for reliability

---

### Schema Caching

**Location**: `~/.config/rds-cli/cache/<profile>/schema.json`
**Format**: JSON, <5ms lookups
**Fuzzy Search**: Levenshtein distance (strsim), max 3, top 3 suggestions

**Key Decision**: JSON serialization avoids DB roundtrips

---

### Query Validation

`sqlparser` with DB dialects for SQL parsing and validation:
- **auto LIMIT**: Injects default_limit when no LIMIT specified
- **max_limit enforcement**: Rejects queries exceeding max_limit
- **allowed_operations**: Dynamically validates against configured operations (SELECT, EXPLAIN, INSERT, etc.)

### Named Queries

Storage: `.rds-cli.toml`, Parameters: `:param_name` (regex), `toml_edit` for Git diffs

---

## Patterns

**CliContext**: Single config loading point in all handlers (`CliContext::load(cli)`)
**Error Handling**: `anyhow::Result` everywhere, no custom error types
**Handler Pattern**: `async fn handle_*(args, cli: &Cli) -> Result<()>`

---

## Adding Features

**New Database**: Implement `Database` trait (4 methods), register in `src/db/mod.rs` factory
**New Command**: Add to `Command` enum, create handler with `CliContext::load(cli)`
**Config Field**: Add to `ApplicationConfig`, update `merge()` logic

---

## Common Issues

**Cache Not Found**: Run `rds-cli refresh`
**Query Validation Failed**: Modify `allowed_operations` in profile safety config
**Connection Failed**: Check `rds-cli secret get <profile>`, run `--verbose` flag
**Decryption Failed**: `rds-cli secret reset` then re-set passwords
**Master Key Lost**: Same as decryption failed (key is per-user, not shared)

---

## Key Files

- `src/main.rs`: CLI handlers, CliContext pattern, secret handler
- `src/config.rs`: 5-level hierarchy, merge logic, encrypted password auto-decryption
- `src/crypto.rs`: ChaCha20-Poly1305 encryption/decryption
- `src/secret.rs`: Master key management (file-based storage)
- `src/cache.rs`: Schema caching, fuzzy search
- `src/validator.rs`: SQL parsing, LIMIT injection
- `src/query_manager.rs`: Named queries, parameter extraction, toml_edit
- `src/db/mod.rs`: Database trait, factory
- `src/db/postgres.rs`: PostgreSQL implementation
- `src/db/mysql.rs`: MySQL implementation
- `src/format.rs`: Output formatting (table, json, csv)

---

## Performance

**Binary**: 6.9MB (LTO + strip)
**Cache**: <5ms, **Validation**: <1ms
**Release**: `lto = true`, `strip = true`, `opt-level = 3`

---

For user documentation, see [README.md](README.md).
