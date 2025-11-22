---
name: db-query
description: Query PostgreSQL/MySQL databases via rds-cli. Use when user asks about database schema, tables, queries, or data analysis. Triggers - database, SQL, schema, query, table, PostgreSQL, MySQL, data.
allowed-tools: Bash, Read
---

# Database Query Skill

Execute database queries and explore schemas using `rds-cli`.

## Critical: Global Options BEFORE Command

```bash
# ✅ CORRECT
rds-cli --format json query "SELECT * FROM users"
rds-cli --profile prod schema find order

# ❌ WRONG (will fail)
rds-cli query "SELECT * FROM users" --format json
rds-cli schema find order --profile prod
```

---

## Quick Reference

### Schema Exploration

```bash
# Find tables
rds-cli --format json schema find user

# Table details
rds-cli --format json schema show users

# Relationships
rds-cli schema relationships orders
```

### Execute Queries

```bash
# Ad-hoc query
rds-cli --format json query "SELECT * FROM users WHERE status = 'active'"

# Named query
rds-cli --format json run order_stats
rds-cli --format json run search_user --param email=test@example.com
```

### Manage Named Queries

```bash
# List
rds-cli --format json saved

# Save
rds-cli saved save daily_stats "SELECT DATE(created_at), COUNT(*) FROM orders GROUP BY 1" -d "Daily order count"

# Delete
rds-cli saved delete old_query
```

---

## AI Agent Workflow

1. **Explore schema**: `rds-cli --format json schema find <keyword>`
2. **Inspect table**: `rds-cli --format json schema show <table>`
3. **Execute query**: `rds-cli --format json query "<sql>"` or `run <name>`
4. **Parse JSON**: All output is jq-compatible

---

## Key Features

**Schema Caching**: <5ms lookups, auto-refreshed
**Fuzzy Search**: Suggests similar tables on typo
**Named Queries**: Saved in `.rds-cli.toml`, Git-shared
**Safety**: Auto LIMIT (default 1000), SELECT-only by default
**Multi-format**: `--format json|csv|table`

---

## Named Query Parameters

Auto-detected from `:param_name` syntax:

```bash
# Save with parameter
rds-cli saved save find_user "SELECT * FROM users WHERE email = :email"

# Execute with parameter
rds-cli --format json run find_user --param email=user@example.com
```

---

## Configuration

**Priority**: CLI args > ENV vars > Project config > User config > Defaults

**Password**: Use environment variables only
```bash
export DB_PASSWORD_LOCAL="secret"
export DB_PASSWORD_PRODUCTION="prod-secret"
```

**View config**: `rds-cli config show`

---

## Typical Pattern

```bash
# User asks: "How many active users?"

# 1. Find user tables
rds-cli --format json schema find user

# 2. Check structure
rds-cli --format json schema show users | jq '.columns[] | select(.name | contains("status"))'

# 3. Query
rds-cli --format json query "SELECT COUNT(*) FROM users WHERE status = 'active'"

# 4. Return result to user
```

---

**Detailed reference**: See [reference.md](reference.md) and [examples.md](examples.md)
