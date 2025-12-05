# RDS-CLI Command Reference

Quick reference for all commands.

---

## Global Options

**Must appear BEFORE command**:

```bash
--profile, -p <PROFILE>   Database profile
--format, -f <FORMAT>     Output: table, json, json-pretty, csv
--verbose, -v             Verbose output
```

---

## Commands

### `schema find <PATTERN>`

Find tables matching pattern (case-insensitive):

```bash
rds-cli --format json schema find user
rds-cli --format json schema find order
```

**JSON output**:
```json
{
  "tables": [
    {"name": "users", "columns": 34, "primary_key": ["id"], "foreign_keys": 5}
  ]
}
```

---

### `schema show <TABLE>`

Show table structure:

```bash
rds-cli --format json schema show users
```

**JSON output**:
```json
{
  "name": "users",
  "columns": [
    {
      "name": "id",
      "data_type": "uuid",
      "nullable": false,
      "is_primary_key": true,
      "is_foreign_key": false
    }
  ]
}
```

**Fuzzy search**: Suggests similar table names if not found.

---

### `schema relationships <TABLE>`

Show foreign key relationships:

```bash
rds-cli schema relationships orders
rds-cli schema relationships orders --summary
```

**Summary**:
```
Outbound (Foreign Keys): 2
Inbound (Referenced By): 1
```

---

### `query <SQL>`

Execute SQL query:

```bash
rds-cli --format json query "SELECT * FROM users WHERE status = 'active'"
rds-cli --format csv query "SELECT * FROM products"
```

**Safety**:
- Only SELECT allowed (default)
- Auto LIMIT injection (default: 1000)
- Query timeout (default: 10s)

**JSON output**:
```json
{
  "columns": ["id", "email", "status"],
  "rows": [
    ["1", "user@example.com", "active"]
  ],
  "rows_affected": 1
}
```

---

### `run <NAME> [--arg k=v]`

Execute saved query:

```bash
rds-cli --format json run order_stats
rds-cli --format json run search_user --arg email=user@example.com
rds-cli --format json run user_orders --arg user_id=123 --arg status=PENDING
```

**Parameter format**: `key=value` (no spaces)

---

### `saved`

List saved queries:

```bash
rds-cli saved
rds-cli saved --verbose
rds-cli --format json saved
```

**JSON output**:
```json
{
  "queries": [
    {
      "name": "order_stats",
      "description": "Order statistics",
      "params": []
    }
  ]
}
```

---

### `saved save <NAME> <SQL> [-d DESC]`

Save query:

```bash
rds-cli saved save order_stats "SELECT status, COUNT(*) FROM orders GROUP BY status"
rds-cli saved save order_stats "SELECT ..." -d "Order statistics"
rds-cli saved save search_user "SELECT * FROM users WHERE email = :email" -d "Find by email"
```

**Parameters**: Auto-detected from `:param_name`

**Saved to**: `.rds-cli.toml` (project-level)

---

### `saved delete <NAME>`

Delete saved query:

```bash
rds-cli saved delete old_query
```

---

### `saved show <NAME>`

Show query details:

```bash
rds-cli saved show order_stats
```

Output:
```
Query: order_stats
Description: Order statistics

SQL:
SELECT status, COUNT(*) FROM orders GROUP BY status
```

---

### `refresh`

Refresh schema cache:

```bash
rds-cli refresh
rds-cli --profile production refresh
```

---

### `config init`

Create sample config:

```bash
rds-cli config init
```

Creates: `~/.config/rds-cli/application.toml`

---

### `config show`

Display configuration:

```bash
rds-cli config show
```

---

### `config path`

Print config file path:

```bash
rds-cli config path
```

---

### `config edit`

Edit config with $EDITOR:

```bash
rds-cli config edit
```

---

## Configuration

### Priority

```
CLI args > ENV vars > .rds-cli.toml > ~/.config/rds-cli/application.toml > Defaults
```

### Environment Variables

```bash
export DB_PASSWORD_LOCAL="secret"
export DB_PASSWORD_PRODUCTION="prod-secret"
```

Format: `DB_PASSWORD_<PROFILE>` (uppercase)

---

## Output Formats

**table** (default): Human-readable table
**json**: Compact JSON
**json-pretty**: Pretty-printed JSON
**csv**: CSV format

---

## Exit Codes

- `0`: Success
- `1`: Error (config, validation, execution)
