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

## Profile Selection

**Default**: Uses `default_profile: "local"` from config if `--profile` not specified.

**When to specify**:
- User mentions environment: "production", "staging", "dev" → Use `--profile <env>`
- User says "DB" without context → Clarify or use default

```bash
# Specific environment
rds-cli --profile prod --format json query "SELECT COUNT(*) FROM orders"

# Default (uses "local" or configured default_profile)
rds-cli --format json query "SELECT COUNT(*) FROM users"
```

---

## Quick Reference

### Schema Exploration

```bash
# Find tables (default profile)
rds-cli --format json schema find user

# Production environment
rds-cli --profile prod --format json schema show orders

# Relationships
rds-cli schema relationships orders
```

### Execute Queries

```bash
# Ad-hoc query (default)
rds-cli --format json query "SELECT * FROM users WHERE status = 'active'"

# Production query
rds-cli --profile prod --format json query "SELECT * FROM orders WHERE created_at > NOW() - INTERVAL '1 day'"

# Named query with profile
rds-cli --profile staging --format json run order_stats
rds-cli --format json run search_user --arg email=test@example.com
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

1. **Determine profile**: Check if user mentioned environment (prod/staging/dev)
2. **Explore schema**: `rds-cli [--profile <env>] --format json schema find <keyword>`
3. **Inspect table**: `rds-cli [--profile <env>] --format json schema show <table>`
4. **Execute query**: `rds-cli [--profile <env>] --format json query "<sql>"` or `run <name>`
5. **Parse JSON**: All output is jq-compatible

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
rds-cli --format json run find_user --arg email=user@example.com
```

---

## Configuration

**Priority**: CLI args > Encrypted password > ENV vars > Project config > User config

**Password** (encrypted, Git-safe):
```bash
rds-cli secret set local
rds-cli secret set prod
```

**Fallback** (environment variable):
```bash
export DB_PASSWORD_LOCAL="secret"
export DB_PASSWORD_PROD="secret"
```

---

## Typical Pattern

### Example 1: Default Environment
```bash
# User asks: "How many active users?"

# 1. Find user tables (uses default profile)
rds-cli --format json schema find user

# 2. Check structure
rds-cli --format json schema show users | jq '.columns[] | select(.name | contains("status"))'

# 3. Query
rds-cli --format json query "SELECT COUNT(*) FROM users WHERE status = 'active'"

# 4. Return result to user
```

### Example 2: Specific Environment
```bash
# User asks: "How many production orders today?"

# 1. Find tables (production DB)
rds-cli --profile prod --format json schema find order

# 2. Check structure
rds-cli --profile prod --format json schema show orders | jq '.columns[] | select(.name | contains("created"))'

# 3. Query with profile
rds-cli --profile prod --format json query "SELECT COUNT(*) FROM orders WHERE DATE(created_at) = CURRENT_DATE"

# 4. Return result to user
```
