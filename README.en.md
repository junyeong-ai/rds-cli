# RDS CLI

[![CI](https://github.com/junyeong-ai/rds-cli/workflows/CI/badge.svg)](https://github.com/junyeong-ai/rds-cli/actions)
[![Lint](https://github.com/junyeong-ai/rds-cli/workflows/Lint/badge.svg)](https://github.com/junyeong-ai/rds-cli/actions)
[![Rust](https://img.shields.io/badge/rust-1.91.1%2B%20(2024%20edition)-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-0.1.0-blue?style=flat-square)](https://github.com/junyeong-ai/rds-cli/releases)

> **🌐 [한국어](README.md)** | **English**

---

> **⚡ Fast and Safe Database CLI for PostgreSQL/MySQL**
>
> - 🚀 **Blazing Fast** (Rust-based, <5ms schema lookup)
> - 🔒 **Production Safe** (Auto LIMIT, read-only mode)
> - 📝 **Team Collaboration** (Git-versioned Named Queries)
> - 🔍 **Smart Search** (Fuzzy matching, auto-complete)

---

## Why RDS CLI?

Traditional DB clients are **slow**, **dangerous**, and **hard to collaborate**.

| Traditional | RDS CLI |
|------------|---------|
| 🐌 Schema lookup every time (100ms+) | ⚡ Cached <5ms lookup |
| ❌ Accidental full table scan | ✅ Auto LIMIT applied |
| 🔓 DELETE possible in production | 🔒 Read-only enforced |
| 📋 Copy-paste complex queries | 📝 Team-shared Named Queries |
| 🤷 "Table not found" on typo | 🔍 Fuzzy search suggestions |

---

## ⚡ Quick Start

```bash
# 1. Install (one-liner)
curl -fsSL https://raw.githubusercontent.com/junyeong-ai/rds-cli/main/scripts/install.sh | bash

# 2. Configure (1 minute)
rds-cli config init
rds-cli config edit  # Enter DB credentials

# 3. Cache schema
export DB_PASSWORD_LOCAL="your-password"
rds-cli refresh

# 4. Start using!
rds-cli schema find user
rds-cli query "SELECT * FROM users"
```

---

## 🎯 Key Features

### 1. Lightning-Fast Schema Exploration

```bash
# Find tables (typos OK)
rds-cli schema show user  # → suggests "users"
rds-cli schema find order # → finds orders, order_items

# Analyze relationships
rds-cli schema relationships orders
```

### 2. Safe Query Execution

```bash
# Auto LIMIT (prevent mistakes)
rds-cli query "SELECT * FROM orders"
# → SELECT * FROM orders LIMIT 1000

# Production read-only
rds-cli --profile prod query "DELETE FROM users"
# → ERROR: Only SELECT queries allowed
```

### 3. Named Queries for Team Collaboration

```bash
# Save to .rds-cli.toml (Git-shared)
rds-cli saved save active_users \
  "SELECT * FROM users WHERE last_login > NOW() - INTERVAL '7 days'"

# Team members run by name
rds-cli run active_users

# Parameterized queries
rds-cli saved save find_user "SELECT * FROM users WHERE email = :email"
rds-cli run find_user --param email=test@example.com
```

### 4. Multiple Output Formats

```bash
# JSON (jq pipeline)
rds-cli --format json query "SELECT status, COUNT(*) FROM orders GROUP BY status" \
  | jq '.rows | map({status: .[0], count: .[1]})'

# CSV (Excel import)
rds-cli --format csv query "SELECT * FROM products" > products.csv
```

---

## 📦 Installation

### Recommended: Prebuilt Binary

```bash
curl -fsSL https://raw.githubusercontent.com/junyeong-ai/rds-cli/main/scripts/install.sh | bash
```

### Cargo

```bash
cargo install rds-cli
```

**Optional**: Install Claude Code Skill to explore databases with natural language.

---

## ⚙️ Configuration

### Priority Order

```
--profile option > DB_PASSWORD_<PROFILE> env > .rds-cli.toml > ~/.config/rds-cli/config.toml
```

### Minimal Config Example

**~/.config/rds-cli/config.toml**:

```toml
[defaults]
default_profile = "local"

[profiles.local]
type = "postgresql"
host = "localhost"
port = 5432
user = "myuser"
database = "mydb"

[profiles.local.safety]
default_limit = 1000
allowed_operations = ["SELECT"]
```

**Use environment variables for passwords**:

```bash
export DB_PASSWORD_LOCAL="secret"
export DB_PASSWORD_PRODUCTION="prod-secret"
```

**Team-shared queries** (./.rds-cli.toml, Git-committed):

```toml
[saved_queries.daily_stats]
sql = "SELECT DATE(created_at), COUNT(*) FROM orders GROUP BY 1 ORDER BY 1 DESC LIMIT 7"
description = "Last 7 days order statistics"
```

### Config Commands

```bash
rds-cli config init   # Create config file
rds-cli config edit   # Edit with $EDITOR
rds-cli config show   # Show current config
```

---

## 💡 Practical Usage

### Production Safety Pattern

```bash
# Production: read-only + low LIMIT
[profiles.production.safety]
default_limit = 100
max_limit = 1000
allowed_operations = ["SELECT"]

# Development: flexible
[profiles.dev.safety]
default_limit = 10000
allowed_operations = ["SELECT", "INSERT", "UPDATE", "DELETE"]
```

### Fuzzy Search Usage

```bash
rds-cli schema show user
# ❌ Table 'user' not found
# Did you mean: users, user_roles, user_sessions?
```

### jq Pipeline

```bash
# Extract primary key
rds-cli --format json schema show users | jq '.columns[] | select(.is_primary_key)'

# Table names only
rds-cli --format json schema find order | jq '.tables[].name'
```

---

## 📖 Command Reference

| Command | Description |
|---------|-------------|
| `schema find <pattern>` | Search tables |
| `schema show <table>` | Show table details |
| `schema relationships <table>` | Analyze relationships |
| `query <sql>` | Execute query |
| `run <name> [--param k=v]` | Run named query |
| `saved [save\|delete\|show]` | Manage queries |
| `refresh` | Refresh schema cache |
| `config [init\|edit\|show]` | Manage configuration |

**Common options**: `--profile <name>`, `--format <json|csv|table>`, `--verbose`

---

## 🛠️ Troubleshooting

### "Cache not found" Error

```bash
rds-cli refresh
```

### "Table not found" Error

```bash
rds-cli schema find <pattern>  # Check table name
rds-cli refresh                # Refresh cache
```

### "Failed to connect" Error

```bash
# Check password environment variable
echo $DB_PASSWORD_<PROFILE>

# Test connection
psql -h localhost -U myuser -d mydb  # PostgreSQL
mysql -h localhost -u myuser -p mydb # MySQL
```

---

## 📄 License

MIT OR Apache-2.0

---

**For AI Agents**: [CLAUDE.md](CLAUDE.md)
