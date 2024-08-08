// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expressions::Expression;
use serde_json::Value;
use tracing::debug;
use tree_sitter::Parser;

use crate::configure::context::Context;
use crate::dscerror::DscError;
use crate::functions::FunctionDispatcher;

pub mod expressions;
pub mod functions;

pub struct Statement {
    parser: Parser,
    function_dispatcher: FunctionDispatcher,
}

impl Statement {
    /// Create a new `StatementParser` instance.
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying parser fails to initialize.
    pub fn new() -> Result<Self, DscError> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_dscexpression::language())?;
        let function_dispatcher = FunctionDispatcher::new();
        Ok(Self {
            parser,
            function_dispatcher,
        })
    }

    /// Parse and execute a statement.
    ///
    /// # Arguments
    ///
    /// * `statement` - The statement to parse and execute.
    ///
    /// # Errors
    ///
    /// This function will return an error if the statement fails to parse or execute.
    pub fn parse_and_execute(&mut self, statement: &str, context: &Context) -> Result<Value, DscError> {
        debug!("Parsing statement: {0}", statement);
        let Some(tree) = &mut self.parser.parse(statement, None) else {
            return Err(DscError::Parser(format!("Error parsing statement: {statement}")));
        };
        let root_node = tree.root_node();
        if root_node.is_error() {
            return Err(DscError::Parser(format!("Error parsing statement root: {statement}")));
        }
        let root_node_kind = root_node.kind();
        if root_node_kind != "statement" {
            return Err(DscError::Parser(format!("Invalid statement: {statement}")));
        }
        let Some(child_node) = root_node.named_child(0) else {
            return Err(DscError::Parser("Child node not found".to_string()));
        };
        if child_node.is_error() {
            return Err(DscError::Parser("Error parsing statement child".to_string()));
        }
        let kind = child_node.kind();
        let statement_bytes = statement.as_bytes();
        match kind {
            "stringLiteral" | "bracketInStringLiteral" => {
                let Ok(value) = child_node.utf8_text(statement_bytes) else {
                    return Err(DscError::Parser("Error parsing string literal".to_string()));
                };
                debug!("Parsing string literal: {0}", value.to_string());
                Ok(Value::String(value.to_string()))
            },
            "escapedStringLiteral" => {
                // need to remove the first character: [[ => [
                let Ok(value) = child_node.utf8_text(statement_bytes) else {
                    return Err(DscError::Parser("Error parsing escaped string literal".to_string()));
                };
                debug!("Parsing escaped string literal: {0}", value[1..].to_string());
                Ok(Value::String(value[1..].to_string()))
            },
            "expression" => {
                debug!("Parsing expression");
                let expression = Expression::new(statement_bytes, &child_node)?;
                Ok(expression.invoke(&self.function_dispatcher, context)?)
            },
            _ => {
                Err(DscError::Parser(format!("Unknown expression type {0}", child_node.kind())))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_literal() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("this is a string", &Context::new()).unwrap();
        assert_eq!(result, "this is a string");
    }

    #[test]
    fn bracket_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[[this is a string]", &Context::new()).unwrap();
        assert_eq!(result, "[this is a string]");
    }

    #[test]
    fn bracket_in_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[this] is a string", &Context::new()).unwrap();
        assert_eq!(result, "[this] is a string");
    }

    #[test]
    fn invalid_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[invalid()]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn nonquoted_string_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat(abc)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn missing_endquote_string_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('abc)]", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn empty_parameter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[concat('abc', , 'def')]", &Context::new());
        assert!(result.is_err());
    }
}
