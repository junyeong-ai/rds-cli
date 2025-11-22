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
