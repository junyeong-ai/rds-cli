use anyhow::Result;
use sqlparser::ast::{Expr, LimitClause, Query, Statement, Value};
use sqlparser::dialect::{Dialect, MySqlDialect, PostgreSqlDialect};
use sqlparser::parser::Parser;

use crate::config::SafetyPolicy;

pub struct QueryValidator {
    policy: SafetyPolicy,
    dialect: Box<dyn Dialect>,
}

impl QueryValidator {
    pub fn new(policy: SafetyPolicy, db_type: &str) -> Self {
        let dialect: Box<dyn Dialect> = match db_type {
            "postgresql" => Box::new(PostgreSqlDialect {}),
            "mysql" => Box::new(MySqlDialect {}),
            _ => Box::new(PostgreSqlDialect {}),
        };

        Self { policy, dialect }
    }

    pub fn validate(&self, sql: &str) -> Result<String> {
        let statements = Parser::parse_sql(&*self.dialect, sql)?;

        if statements.is_empty() {
            anyhow::bail!("No SQL statement provided");
        }

        for statement in &statements {
            self.validate_statement_type(statement)?;
        }

        self.apply_limit_policy(sql, &statements)
    }

    fn validate_statement_type(&self, statement: &Statement) -> Result<()> {
        let stmt_type = match statement {
            Statement::Query(_) => "SELECT",
            Statement::Explain { .. } => "EXPLAIN",
            Statement::ShowTables { .. } | Statement::ShowColumns { .. } => "SHOW",
            Statement::Insert(_) => "INSERT",
            Statement::Update { .. } => "UPDATE",
            Statement::Delete(_) => "DELETE",
            Statement::CreateTable { .. } => "CREATE",
            Statement::Drop { .. } => "DROP",
            Statement::AlterTable { .. } => "ALTER",
            Statement::Truncate { .. } => "TRUNCATE",
            _ => {
                anyhow::bail!("Unsupported statement type");
            }
        };

        let is_allowed = self
            .policy
            .allowed_operations
            .iter()
            .any(|op| op.eq_ignore_ascii_case(stmt_type));

        if !is_allowed {
            anyhow::bail!(
                "Operation '{}' not allowed. Permitted: {:?}",
                stmt_type,
                self.policy.allowed_operations
            );
        }

        Ok(())
    }

    fn apply_limit_policy(&self, sql: &str, statements: &[Statement]) -> Result<String> {
        // Only apply LIMIT policy to SELECT queries
        let is_select = statements.iter().all(|s| matches!(s, Statement::Query(_)));

        if !is_select {
            return Ok(sql.to_string());
        }

        if let Some(user_limit) = self.extract_limit(statements) {
            if user_limit > self.policy.max_limit as u64 {
                anyhow::bail!(
                    "LIMIT {} exceeds maximum allowed ({})",
                    user_limit,
                    self.policy.max_limit
                );
            }
            Ok(sql.to_string())
        } else {
            Ok(format!(
                "{} LIMIT {}",
                sql.trim_end_matches(';'),
                self.policy.default_limit
            ))
        }
    }

    fn extract_limit(&self, statements: &[Statement]) -> Option<u64> {
        for statement in statements {
            if let Statement::Query(query) = statement {
                return self.extract_limit_from_query(query);
            }
        }
        None
    }

    fn extract_limit_from_query(&self, query: &Query) -> Option<u64> {
        // Check for LIMIT in the limit_clause
        if let Some(limit_clause) = &query.limit_clause {
            match limit_clause {
                LimitClause::LimitOffset { limit, .. } => {
                    if let Some(limit_expr) = limit {
                        return self.extract_number_from_expr(limit_expr);
                    }
                }
                LimitClause::OffsetCommaLimit { limit, .. } => {
                    return self.extract_number_from_expr(limit);
                }
            }
        }

        None
    }

    fn extract_number_from_expr(&self, expr: &Expr) -> Option<u64> {
        match expr {
            Expr::Value(value_with_span) => match &value_with_span.value {
                Value::Number(n, _) => n.parse().ok(),
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy() -> SafetyPolicy {
        SafetyPolicy {
            default_limit: 1000,
            max_limit: 10000,
            timeout_seconds: 10,
            allowed_operations: vec!["SELECT".to_string()],
        }
    }

    fn create_policy_with_ops(ops: Vec<&str>) -> SafetyPolicy {
        SafetyPolicy {
            default_limit: 100,
            max_limit: 1000,
            timeout_seconds: 10,
            allowed_operations: ops.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_validate_adds_limit() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("SELECT * FROM users").unwrap();
        assert_eq!(result, "SELECT * FROM users LIMIT 1000");
    }

    #[test]
    fn test_validate_keeps_existing_limit() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("SELECT * FROM users LIMIT 50").unwrap();
        assert_eq!(result, "SELECT * FROM users LIMIT 50");
    }

    #[test]
    fn test_validate_trims_semicolon() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("SELECT * FROM users;").unwrap();
        assert_eq!(result, "SELECT * FROM users LIMIT 1000");
    }

    #[test]
    fn test_validate_rejects_delete() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("DELETE FROM users WHERE id = 1");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Operation 'DELETE' not allowed")
        );
    }

    #[test]
    fn test_validate_rejects_update() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("UPDATE users SET status = 'active'");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Operation 'UPDATE' not allowed")
        );
    }

    #[test]
    fn test_validate_rejects_insert() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("INSERT INTO users (name) VALUES ('test')");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Operation 'INSERT' not allowed")
        );
    }

    #[test]
    fn test_validate_empty_sql() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No SQL statement provided")
        );
    }

    #[test]
    fn test_validate_respects_max_limit() {
        let policy = SafetyPolicy {
            default_limit: 100,
            max_limit: 1000,
            timeout_seconds: 10,
            allowed_operations: vec!["SELECT".to_string()],
        };
        let validator = QueryValidator::new(policy, "postgresql");

        // max_limit exceeded -> error
        let result = validator.validate("SELECT * FROM users LIMIT 5000");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("exceeds maximum allowed")
        );

        // within max_limit -> ok
        let result = validator.validate("SELECT * FROM users LIMIT 500");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_respects_allowed_operations() {
        let policy = create_policy_with_ops(vec!["SELECT", "EXPLAIN"]);
        let validator = QueryValidator::new(policy, "postgresql");

        // SELECT allowed
        assert!(validator.validate("SELECT * FROM users").is_ok());

        // EXPLAIN allowed
        assert!(validator.validate("EXPLAIN SELECT * FROM users").is_ok());

        // DELETE not allowed
        let result = validator.validate("DELETE FROM users");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Operation 'DELETE' not allowed")
        );
    }

    #[test]
    fn test_validate_allows_write_operations_when_configured() {
        let policy = create_policy_with_ops(vec!["SELECT", "INSERT", "UPDATE", "DELETE"]);
        let validator = QueryValidator::new(policy, "postgresql");

        assert!(validator.validate("SELECT * FROM users").is_ok());
        assert!(
            validator
                .validate("INSERT INTO users (name) VALUES ('test')")
                .is_ok()
        );
        assert!(validator.validate("UPDATE users SET name = 'test'").is_ok());
        assert!(validator.validate("DELETE FROM users WHERE id = 1").is_ok());
    }

    #[test]
    fn test_validate_case_insensitive_operations() {
        let policy = create_policy_with_ops(vec!["select", "SELECT"]);
        let validator = QueryValidator::new(policy, "postgresql");

        assert!(validator.validate("SELECT * FROM users").is_ok());
        assert!(validator.validate("select * from users").is_ok());
    }
}
