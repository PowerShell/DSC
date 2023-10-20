// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::dscerror::DscError;
use crate::functions::FunctionDispatcher;
use tree_sitter::Parser;
use expressions::Expression;

pub mod expressions;
pub mod functions;

pub struct StatementParser {
    parser: Parser,
    function_dispatcher: FunctionDispatcher,
}

impl StatementParser {
    pub fn new() -> Result<Self, DscError> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_dscexpression::language())?;
        let function_dispatcher = FunctionDispatcher::new();
        Ok(Self {
            parser,
            function_dispatcher,
        })
    }

    pub fn parse_and_execute(&mut self, statement: &str) -> Result<String, DscError> {
        let tree = &mut self.parser.parse(statement, None).unwrap();
        let root_node = tree.root_node();
        if root_node.is_error() {
            return Err(DscError::Parser("Error parsing statement root".to_string()));
        }
        let root_node_kind = root_node.kind();
        if root_node_kind != "statement" {
            return Err(DscError::Parser("Invalid statement".to_string()));
        }
        let Some(child_node) = root_node.named_child(0) else {
            return Err(DscError::Parser("Child node not found".to_string()));
        };
        if child_node.is_error() {
            return Err(DscError::Parser("Error parsing statement".to_string()));
        }
        let kind = child_node.kind();
        match kind {
            "stringLiteral" | "bracketInStringLiteral" => {
                let value = child_node.utf8_text(statement.as_bytes()).unwrap();
                Ok(value.to_string())
            },
            "escapedStringLiteral" => {
                // need to remove the first character: [[ => [
                let value = child_node.utf8_text(statement.as_bytes()).unwrap();
                Ok(value[1..].to_string())
            },
            "expression" => {
                let expression = Expression::new(&self.function_dispatcher, statement, &child_node)?;
                Ok(expression.invoke()?)
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
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("this is a string").unwrap();
        assert_eq!(result, "this is a string");
    }

    #[test]
    fn bracket_string() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[[this is a string]").unwrap();
        assert_eq!(result, "[this is a string]");
    }

    #[test]
    fn bracket_in_string() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[this] is a string").unwrap();
        assert_eq!(result, "[this] is a string");
    }

    #[test]
    fn invalid_function() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[invalid()]");
        assert!(result.is_err());
    }

    #[test]
    fn nonquoted_string_parameter() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat(abc)]");
        assert!(result.is_err());
    }

    #[test]
    fn missing_endquote_string_parameter() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat('abc)]");
        assert!(result.is_err());
    }

    #[test]
    fn empty_parameter() {
        let mut parser = StatementParser::new().unwrap();
        let result = parser.parse_and_execute("[concat('abc', , 'def')]");
        assert!(result.is_err());
    }
}
