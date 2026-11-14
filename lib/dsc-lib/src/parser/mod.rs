// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use expressions::Expression;
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;
use tree_sitter::Parser;

use crate::configure::context::{Context, ProcessMode};
use crate::dscerror::DscError;
use crate::functions::FunctionDispatcher;

pub mod expressions;
pub mod functions;

pub struct Statement {
    parser: Parser,
    function_dispatcher: FunctionDispatcher,
}

/// The result of parsing a statement without evaluating it.
#[derive(Clone, Debug)]
pub enum ParsedStatement {
    /// The statement is a plain string that doesn't contain an expression.
    Literal(String),
    /// The statement started with `[[`; the value has the escape character removed.
    Escaped(String),
    /// The statement is an expression that can be invoked.
    Expression(Expression),
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
        if context.process_mode == ProcessMode::NoExpressionEvaluation {
            debug!("{}", t!("parser.skippingExpressionProcessing"));
            return Ok(Value::String(statement.to_string()));
        }

        // if statement is empty or just whitespace, return as string without parsing
        if statement.trim().is_empty() {
            return Ok(Value::String(statement.to_string()));
        }

        match self.parse(statement)? {
            ParsedStatement::Literal(value) | ParsedStatement::Escaped(value) => Ok(Value::String(value)),
            ParsedStatement::Expression(expression) => expression.invoke(&self.function_dispatcher, context),
        }
    }

    /// Parse a statement without evaluating it.
    ///
    /// Use this to inspect the expression tree of a statement, for example to find which
    /// functions it calls, without needing a [`Context`] and without side effects.
    ///
    /// # Arguments
    ///
    /// * `statement` - The statement to parse.
    ///
    /// # Returns
    ///
    /// The parsed statement: a literal, an escaped literal, or an expression tree.
    ///
    /// # Errors
    ///
    /// This function will return an error if the statement fails to parse.
    pub fn parse(&mut self, statement: &str) -> Result<ParsedStatement, DscError> {
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
        let mut parsed = ParsedStatement::Literal(statement.to_string());
        for child_node in root_node.named_children(&mut cursor) {
            if child_node.is_error() {
                return Err(DscError::Parser(t!("parser.failedToParse", statement = statement).to_string()));
            }

            parsed = match child_node.kind() {
                "stringLiteral" => {
                    let Ok(value) = child_node.utf8_text(statement_bytes) else {
                        return Err(DscError::Parser(t!("parser.failedToParseStringLiteral").to_string()));
                    };
                    debug!("{}", t!("parser.parsingStringLiteral", value = value.to_string()));
                    ParsedStatement::Literal(value.to_string())
                },
                "escapedStringLiteral" => {
                    // need to remove the first character: [[ => [
                    let Ok(value) = child_node.utf8_text(statement_bytes) else {
                        return Err(DscError::Parser(t!("parser.failedToParseEscapedStringLiteral").to_string()));
                    };
                    debug!("{}", t!("parser.parsingEscapedStringLiteral", value = value[1..].to_string()));
                    ParsedStatement::Escaped(value[1..].to_string())
                },
                "expression" => {
                    debug!("{}", t!("parser.parsingExpression"));
                    ParsedStatement::Expression(Expression::new(statement_bytes, &child_node)?)
                },
                _ => {
                    return Err(DscError::Parser(t!("parser.unknownExpressionType", kind = child_node.kind()).to_string()));
                }
            };
        }

        Ok(parsed)
    }

    /// Get the function dispatcher used to evaluate expressions.
    #[must_use]
    pub fn function_dispatcher(&self) -> &FunctionDispatcher {
        &self.function_dispatcher
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
    fn multiline_bracket_string() {
        let mut parser = Statement::new().unwrap();
        let statement = "[[Version]\nSignature=\"$Windows NT$\"\n\n[PolicyStatementExtension]\nPolicies=InternalPolicy\n";
        let result = parser.parse_and_execute(statement, &Context::new()).unwrap();
        assert_eq!(
            result,
            "[Version]\nSignature=\"$Windows NT$\"\n\n[PolicyStatementExtension]\nPolicies=InternalPolicy\n"
        );
    }

    #[test]
    fn bracket_in_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[this] is a string", &Context::new());
        assert!(result.is_err());
    }

    #[test]
    fn parse_literal_without_evaluating() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse("plain text").unwrap();
        assert!(matches!(result, ParsedStatement::Literal(value) if value == "plain text"));
    }

    #[test]
    fn parse_escaped_literal_without_evaluating() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse("[[not an expression]").unwrap();
        assert!(matches!(result, ParsedStatement::Escaped(value) if value == "[not an expression]"));
    }

    #[test]
    fn parse_expression_without_evaluating() {
        let mut parser = Statement::new().unwrap();
        // `parameters('x')` would fail to evaluate without a context, but parsing must succeed
        let result = parser.parse("[parameters('x').member]").unwrap();
        let ParsedStatement::Expression(expression) = result else {
            panic!("expected an expression");
        };
        assert_eq!(expression.function().name(), "parameters");
        assert_eq!(expression.function().args().map(<[_]>::len), Some(1));
        assert_eq!(expression.accessors().len(), 1);
    }

    #[test]
    fn parse_invalid_expression_fails() {
        let mut parser = Statement::new().unwrap();
        assert!(parser.parse("[concat('abc)]").is_err());
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
