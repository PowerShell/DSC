// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expressions::Expression;
use rust_i18n::t;
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
        parser.set_language(&tree_sitter_dscexpression::LANGUAGE.into())?;
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
        debug!("{}", t!("parser.parsingStatement", statement = statement));
        if !context.process_expressions {
            debug!("{}", t!("parser.skippingExpressionProcessing"));
            return Ok(Value::String(statement.to_string()));
        }

        let Some(tree) = &mut self.parser.parse(statement, None) else {
            return Err(DscError::Parser(t!("parser.failedToParse", statement = statement).to_string()));
        };
        let root_node = tree.root_node();
        if root_node.is_error() {
            return Err(DscError::Parser(t!("parser.failedToParseRoot", statement = statement).to_string()));
        }
        if root_node.kind() != "statement" {
            return Err(DscError::Parser(t!("parser.invalidStatement", statement = statement).to_string()));
        }
        let statement_bytes = statement.as_bytes();
        let mut cursor = root_node.walk();
        let mut return_value = Value::Null;
        for child_node in root_node.named_children(&mut cursor) {
            if child_node.is_error() {
                return Err(DscError::Parser(t!("parser.failedToParse", statement = statement).to_string()));
            }

            match child_node.kind() {
                "stringLiteral" => {
                    let Ok(value) = child_node.utf8_text(statement_bytes) else {
                        return Err(DscError::Parser(t!("parser.failedToParseStringLiteral").to_string()));
                    };
                    debug!("{}", t!("parser.parsingStringLiteral", value = value.to_string()));
                    return_value = Value::String(value.to_string());
                },
                "escapedStringLiteral" => {
                    // need to remove the first character: [[ => [
                    let Ok(value) = child_node.utf8_text(statement_bytes) else {
                        return Err(DscError::Parser(t!("parser.failedToParseEscapedStringLiteral").to_string()));
                    };
                    debug!("{}", t!("parser.parsingEscapedStringLiteral", value = value[1..].to_string()));
                    return_value = Value::String(value[1..].to_string());
                },
                "expression" => {
                    debug!("{}", t!("parser.parsingExpression"));
                    let expression = Expression::new(statement_bytes, &child_node)?;
                    return_value = expression.invoke(&self.function_dispatcher, context)?;
                },
                _ => {
                    return Err(DscError::Parser(t!("parser.unknownExpressionType", kind = child_node.kind()).to_string()));
                }
            }
        }

        Ok(return_value)
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
        let result = parser.parse_and_execute("[this] is a string", &Context::new());
        assert!(result.is_err());
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
