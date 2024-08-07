// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde_json::Value;
use tracing::{debug, trace};
use tree_sitter::Node;

use crate::configure::context::Context;
use crate::dscerror::DscError;
use crate::functions::FunctionDispatcher;
use crate::parser::functions::Function;

#[derive(Clone)]
pub enum Accessor {
    Member(String),
    Index(Value),
}

#[derive(Clone)]
pub struct Expression {
    function: Function,
    accessors: Vec<Accessor>,
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
        debug!("Parsing function '{:?}'", function);
        let function = Function::new(statement_bytes, &function)?;
        let mut accessors = Vec::<Accessor>::new();
        if let Some(accessor) = expression.child_by_field_name("accessor") {
            debug!("Parsing accessor '{:?}'", accessor);
            if accessor.is_error() {
                return Err(DscError::Parser("Error parsing accessor".to_string()));
            }
            let mut cursor = accessor.walk();
            for accessor in accessor.named_children(&mut cursor) {
                if accessor.is_error() {
                    return Err(DscError::Parser("Error parsing accessor".to_string()));
                }
                let accessor_kind = accessor.kind();
                let value = match accessor_kind {
                    "memberAccess" => {
                        debug!("Parsing member accessor '{:?}'", accessor);
                        let Some(member_name) = accessor.child_by_field_name("name") else {
                            return Err(DscError::Parser("Member name not found".to_string()));
                        };
                        let member = member_name.utf8_text(statement_bytes)?;
                        Accessor::Member(member.to_string())
                    },
                    "index" => {
                        debug!("Parsing index accessor '{:?}'", accessor);
                        let Some(index_value) = accessor.child_by_field_name("indexValue") else {
                            return Err(DscError::Parser("Index value not found".to_string()));
                        };
                        match index_value.kind() {
                            "number" => {
                                let value = index_value.utf8_text(statement_bytes)?;
                                let value = serde_json::from_str(value)?;
                                Accessor::Index(value)
                            },
                            "expression" => {
                                return Err(DscError::Parser("Expression index not supported".to_string()));
                            },
                            _ => {
                                return Err(DscError::Parser(format!("Invalid accessor kind: '{:?}'", accessor_kind)));
                            },
                        }
                    },
                    _ => {
                        return Err(DscError::Parser(format!("Invalid accessor kind: '{:?}'", accessor_kind)));
                    },
                };
                accessors.push(value);
            }
        }

        Ok(Expression {
            function,
            accessors,
        })
    }

    /// Invoke the expression.
    ///
    /// # Arguments
    ///
    /// * `function_dispatcher` - The function dispatcher to use.
    /// * `context` - The context to use.
    ///
    /// # Returns
    ///
    /// The result of the expression.
    ///
    /// # Errors
    ///
    /// This function will return an error if the expression fails to execute.
    pub fn invoke(&self, function_dispatcher: &FunctionDispatcher, context: &Context) -> Result<Value, DscError> {
        let result = self.function.invoke(function_dispatcher, context)?;
        trace!("Function result: '{:?}'", result);
        if self.accessors.len() > 0 {
            debug!("Evaluating accessors");
            let mut value = result;
            for accessor in &self.accessors {
                match accessor {
                    Accessor::Member(member) => {
                        if !value.is_object() {
                            return Err(DscError::Parser("Member access on non-object value".to_string()));
                        }
                        if let Some(object) = value.as_object() {
                            if !object.contains_key(member) {
                                return Err(DscError::Parser(format!("Member '{:?}' not found", member)));
                            }
                            value = object[member].clone();
                        }
                    },
                    Accessor::Index(index) => {
                        if !value.is_array() {
                            return Err(DscError::Parser("Index access on non-array value".to_string()));
                        }
                        if let Some(array) = value.as_array() {
                            if !index.is_number() {
                                return Err(DscError::Parser("Index is not a number".to_string()));
                            }
                            let index = index.as_u64().unwrap() as usize;
                            if index >= array.len() {
                                return Err(DscError::Parser("Index out of bounds".to_string()));
                            }
                            value = array[index].clone();
                        }
                    },
                }
            }

            Ok(value)
        }
        else {
            Ok(result)
        }
    }
}
