// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use tree_sitter::Node;

use crate::dscerror::DscError;
use crate::functions::FunctionDispatcher;
use crate::parser::functions::{Function, FunctionResult};

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
    pub fn new(function_dispatcher: &FunctionDispatcher, statement_bytes: &[u8], expression: &Node) -> Result<Self, DscError> {
        let Some(function) = expression.child_by_field_name("function") else {
            return Err(DscError::Parser("Function node not found".to_string()));
        };
        let function = Function::new(function_dispatcher, statement_bytes, &function)?;
        let member_access = if let Some(member_access) = expression.child_by_field_name("members") {
            if member_access.is_error() {
                return Err(DscError::Parser("Error parsing dot-notation".to_string()));
            }
            let mut result = vec![];
            let mut cursor = member_access.walk();
            for member in member_access.children(&mut cursor) {
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
    pub fn invoke(&self, function_dispatcher: &FunctionDispatcher) -> Result<String, DscError> {
        let result = self.function.invoke(function_dispatcher)?;
        if let Some(member_access) = &self.member_access {
            match result {
                FunctionResult::String(_) => {
                    Err(DscError::Parser("Member access on string not supported".to_string()))
                },
                FunctionResult::Object(object) => {
                    let mut value = object;
                    if !value.is_object() {
                        return Err(DscError::Parser(format!("Member access on non-object value '{value}'")));
                    }
                    for member in member_access {
                        value = value[member].clone();
                    }
                    Ok(value.to_string())
                }
            }
        }
        else {
            match result {
                FunctionResult::String(value) => Ok(value),
                FunctionResult::Object(object) => Ok(object.to_string()),
            }
        }
    }
}
