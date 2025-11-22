# RDS-CLI Usage Examples

Practical patterns for AI agents and automation.

---

## Example 1: Schema Exploration

**User asks**: "Show me the users table structure"

```bash
# Find table
rds-cli --format json schema find user | jq '.tables[].name'
# ["users", "user_roles", "user_sessions"]

# Inspect structure
rds-cli --format json schema show users | jq '.columns[] | {name, type: .data_type, nullable}'
# [{"name": "id", "type": "uuid", "nullable": false}, ...]

# Check relationships
rds-cli schema relationships users --summary
# Outbound: 2, Inbound: 5
```

---

## Example 2: Natural Language Query Execution

**User asks**: "How many pending orders from users created this week?"

```bash
# 1. Find relevant tables
rds-cli --format json schema find order
rds-cli --format json schema find user

# 2. Check join column
rds-cli --format json schema show orders | jq '.columns[] | select(.name | contains("user"))'
# {"name": "user_id", "is_foreign_key": true}

# 3. Execute query
rds-cli --format json query "
  SELECT COUNT(*) as pending_count
  FROM orders o
  JOIN users u ON o.user_id = u.id
  WHERE o.status = 'PENDING'
    AND u.created_at >= CURRENT_DATE - INTERVAL '7 days'
"

# 4. Return: "There are 42 pending orders from users created this week."
```

---

## Example 3: Data Export & Analysis

**Export to CSV for Excel**:

```bash
rds-cli --format csv query "
  SELECT
    DATE(created_at) as date,
    status,
    COUNT(*) as count,
    SUM(total_amount) as revenue
  FROM orders
  WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
  GROUP BY DATE(created_at), status
  ORDER BY date DESC
" > /tmp/order_stats.csv
```

**JSON for API**:

```bash
rds-cli --format json query "SELECT * FROM users WHERE status = 'active'" \
  | jq '{users: [.rows[] | {id: .[0], email: .[1]}]}' \
  > active_users.json
```

**jq Processing**:

```bash
# Count by status
rds-cli --format json query "SELECT status, COUNT(*) FROM orders GROUP BY status" \
  | jq '.rows | map({status: .[0], count: .[1] | tonumber})'

# Extract emails only
rds-cli --format json query "SELECT email FROM users WHERE status = 'active'" \
  | jq -r '.rows[] | .[0]'
```

---

## Example 4: Named Queries for Team Collaboration

**Developer A** saves queries:

```bash
# Save common query
rds-cli saved save active_users \
  "SELECT id, email, last_login FROM users WHERE status = 'active' ORDER BY last_login DESC" \
  -d "Active users by last login"

# Save parameterized query
rds-cli saved save user_orders \
  "SELECT * FROM orders WHERE user_id = :user_id AND created_at >= :start_date" \
  -d "User orders since date"

# Commit to Git
git add .rds-cli.toml
git commit -m "Add user query templates"
```

**Developer B** uses them:

```bash
# Pull and list
git pull
rds-cli --format json saved | jq '.queries[].name'
# ["active_users", "user_orders"]

# Execute
rds-cli --format json run active_users
rds-cli --format json run user_orders --param user_id=123 --param start_date='2025-01-01'
```

---

## Example 5: Multi-Environment Queries

```bash
# Compare counts across environments
for env in dev staging production; do
  echo -n "$env: "
  rds-cli --profile $env --format json query "SELECT COUNT(*) FROM users" \
    | jq -r '.rows[0][0]'
done

# Output:
# dev: 1234
# staging: 5678
# production: 98765
```

---

## Tips & Tricks

**Monitor with watch**:
```bash
watch -n 5 'rds-cli --format json query "SELECT COUNT(*) FROM orders WHERE status = '\''PENDING'\''" | jq -r ".rows[0][0]"'
```

**Parallel queries**:
```bash
for table in users orders products; do
  rds-cli --format json query "SELECT COUNT(*) FROM $table" \
    | jq -r --arg t "$table" '"\($t): \(.rows[0][0])"' &
done
wait
```

**Error handling in scripts**:
```bash
if ! result=$(rds-cli --format json query "SELECT * FROM users LIMIT 1" 2>&1); then
  echo "Query failed: $result"
  exit 1
fi
```
