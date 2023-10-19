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
        let root_node_kind = root_node.kind();
        if root_node_kind != "statement" {
            return Err(DscError::Parser("Invalid statement".to_string()));
        }
        let Some(child_node) = root_node.named_child(0) else {
            return Err(DscError::Parser("Child node not found".to_string()));
        };
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
