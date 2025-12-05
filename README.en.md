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

## Key Features

- **Fast Schema Lookup**: Cached <5ms
- **Safe Queries**: Auto LIMIT, read-only mode
- **Team Collaboration**: Git-versioned Named Queries
- **Encrypted Passwords**: Git safe, no environment variables
- **Smart Search**: Fuzzy matching, auto suggestions

---

## ⚡ Quick Start

```bash
# 1. Install (auto-creates global config)
curl -fsSL https://raw.githubusercontent.com/junyeong-ai/rds-cli/main/scripts/install.sh | bash

# 2. Project setup
cd your-project
rds-cli config init     # Creates .rds-cli.toml
rds-cli config edit     # Enter DB credentials

# 3. Set password (encrypted)
rds-cli secret set local

# 4. Cache schema and use
rds-cli refresh
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

### 3. Encrypted Password Management

```bash
# Set password (encrypted in .rds-cli.toml)
rds-cli secret set production

# Automation
echo "password" | rds-cli secret set production --password-stdin
```

### 4. Named Queries for Team Collaboration

```bash
# Save to .rds-cli.toml (Git-shared)
rds-cli saved save active_users \
  "SELECT * FROM users WHERE last_login > NOW() - INTERVAL '7 days'"

# Team members run by name
rds-cli run active_users

# Parameterized queries
rds-cli saved save find_user "SELECT * FROM users WHERE email = :email"
rds-cli run find_user --arg email=test@example.com
```

### 5. Multiple Output Formats

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

### Configuration Structure

**Global config** (`~/.config/rds-cli/config.toml`, auto-created on install):
```toml
[defaults]
default_profile = "local"
cache_ttl_hours = 24
output_format = "table"
```

**Project config** (`.rds-cli.toml`, created by `config init`):
```toml
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

**Priority**: CLI args > Encrypted password > Environment variable > Project config > Global config

### Password Management

**Recommended: Encrypted Storage**
```bash
rds-cli secret set local
# Encrypted in .rds-cli.toml (Git safe)
```

**Optional: Environment Variable**
```bash
export DB_PASSWORD_LOCAL="secret"
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
rds-cli config path   # Print config file path
```

---

## Production Configuration

```toml
[profiles.production.safety]
default_limit = 100
max_limit = 1000
allowed_operations = ["SELECT"]  # Read-only
```

---

## 📖 Command Reference

| Command | Description |
|---------|-------------|
| `schema find <pattern>` | Search tables |
| `schema show <table>` | Show table details |
| `schema relationships <table>` | Analyze relationships |
| `query <sql>` | Execute query |
| `run <name> [-a k=v]` | Run named query |
| `saved [list\|save\|delete\|show]` | Manage queries |
| `secret set <profile>` | Store encrypted password |
| `secret get <profile>` | Decrypt and print password |
| `secret remove <profile>` | Remove password |
| `secret reset` | Reset master key |
| `refresh` | Refresh schema cache |
| `config [init\|edit\|show\|path]` | Manage configuration |

**Options**: `--profile <name>`, `--format <json|csv|table>`, `--verbose`

---

## Troubleshooting

```bash
# Cache not found
rds-cli refresh

# Connection failed
rds-cli secret get <profile>

# Master key lost
rds-cli secret reset
rds-cli secret set <profile>
```

---

## 📄 License

MIT OR Apache-2.0

---

**For AI Agents**: [CLAUDE.md](CLAUDE.md)
