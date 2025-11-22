use anyhow::Result;
use sqlparser::ast::Statement;
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
            match statement {
                Statement::Query(_) => {}
                _ => {
                    anyhow::bail!("Only SELECT queries allowed. Statement type not permitted");
                }
            }
        }

        let transformed_sql = if !self.has_limit(sql) {
            format!(
                "{} LIMIT {}",
                sql.trim_end_matches(';'),
                self.policy.default_limit
            )
        } else {
            sql.to_string()
        };

        Ok(transformed_sql)
    }

    fn has_limit(&self, sql: &str) -> bool {
        sql.to_uppercase().contains("LIMIT")
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

    #[test]
    fn test_has_limit_true() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        assert!(validator.has_limit("SELECT * FROM users LIMIT 10"));
        assert!(validator.has_limit("SELECT * FROM users limit 10"));
        assert!(validator.has_limit("select * from users LIMIT 10"));
    }

    #[test]
    fn test_has_limit_false() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        assert!(!validator.has_limit("SELECT * FROM users"));
        assert!(!validator.has_limit("SELECT * FROM users WHERE id = 1"));
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
                .contains("Only SELECT queries allowed")
        );
    }

    #[test]
    fn test_validate_rejects_update() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("UPDATE users SET status = 'active'");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_rejects_insert() {
        let validator = QueryValidator::new(create_test_policy(), "postgresql");
        let result = validator.validate("INSERT INTO users (name) VALUES ('test')");
        assert!(result.is_err());
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
}
