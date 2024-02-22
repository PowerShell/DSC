// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::Value;
use tracing::debug;
use tree_sitter::Node;

use crate::configure::context::Context;
use crate::dscerror::DscError;
use crate::functions::FunctionDispatcher;
use crate::parser::functions::Function;

#[derive(Clone)]
pub struct Expression {
    function: Function,
    member_access: Option<Vec<String>>,
}

impl Expression {
    /// Create a new `Expression` instance.
    ///
    /// # Arguments
    ///
    /// * `function_dispatcher` - The function dispatcher to use.
    /// * `statement` - The statement that the expression is part of.
    /// * `expression` - The expression node.
    ///
    /// # Errors
    ///
    /// This function will return an error if the expression node is not valid.
    pub fn new(statement_bytes: &[u8], expression: &Node) -> Result<Self, DscError> {
        let Some(function) = expression.child_by_field_name("function") else {
            return Err(DscError::Parser("Function node not found".to_string()));
        };
        let function = Function::new(statement_bytes, &function)?;
        let member_access = if let Some(members) = expression.child_by_field_name("members") {
            if members.is_error() {
                return Err(DscError::Parser("Error parsing dot-notation".to_string()));
            }
            let mut result = vec![];
            let mut cursor = members.walk();
            for member in members.children(&mut cursor) {
                if member.is_error() {
                    return Err(DscError::Parser("Error parsing dot-notation member".to_string()));
                }
                let value = member.utf8_text(statement_bytes)?;
                result.push(value.to_string());
            }
            Some(result)
        }
        else {
            None
        };
        Ok(Expression {
            function,
            member_access,
        })
    }

    /// Invoke the expression.
    ///
    /// # Errors
    ///
    /// This function will return an error if the expression fails to execute.
    pub fn invoke(&self, function_dispatcher: &FunctionDispatcher, context: &Context) -> Result<Value, DscError> {
        let result = self.function.invoke(function_dispatcher, context)?;
        if let Some(member_access) = &self.member_access {
            debug!("Evaluating member access '{:?}'", member_access);
            if !result.is_object() {
                return Err(DscError::Parser("Member access on non-object value".to_string()));
            }

            let mut value = result;
            for member in member_access {
                if !value.is_object() {
                    return Err(DscError::Parser(format!("Member access '{member}' on non-object value")));
                }

                if let Some(object) = value.as_object() {
                    if !object.contains_key(member) {
                        return Err(DscError::Parser(format!("Member '{member}' not found")));
                    }

                    value = object[member].clone();
                }
            }

            Ok(value)
        }
        else {
            Ok(result)
        }
    }
}
