// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::Value;
use tracing::{debug, trace};
use tree_sitter::Node;

use crate::configure::context::Context;
use crate::configure::parameters::is_secure_value;
use crate::dscerror::DscError;
use crate::functions::FunctionDispatcher;
use crate::parser::functions::Function;

#[derive(Clone, Debug)]
pub enum Accessor {
    Member(String),
    Index(Value),
    IndexExpression(Expression),
}

#[derive(Clone, Debug)]
pub struct Expression {
    function: Function,
    accessors: Vec<Accessor>,
}

fn node_to_string(node: &Node, statement_bytes: &[u8]) -> Result<String, DscError> {
    let text = node.utf8_text(statement_bytes)?;
    Ok(text.to_string())
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
            return Err(DscError::Parser(t!("parser.expression.functionNodeNotFound").to_string()));
        };
        debug!("{}", t!("parser.expression.parsingFunction", name = node_to_string(&function, statement_bytes)? : {:?}));
        let function = Function::new(statement_bytes, &function)?;
        let mut accessors = Vec::<Accessor>::new();
        if let Some(accessor) = expression.child_by_field_name("accessor") {
            debug!("{}", t!("parser.expression.parsingAccessor", name = node_to_string(&accessor, statement_bytes)? : {:?}));
            if accessor.is_error() {
                return Err(DscError::Parser(t!("parser.expression.accessorParsingError").to_string()));
            }
            let mut cursor = accessor.walk();
            for accessor in accessor.named_children(&mut cursor) {
                if accessor.is_error() {
                    return Err(DscError::Parser(t!("parser.expression.accessorParsingError").to_string()));
                }
                let accessor_kind = accessor.kind();
                let value = match accessor_kind {
                    "memberAccess" => {
                        debug!("{}", t!("parser.expression.parsingMemberAccessor", name = node_to_string(&accessor, statement_bytes)? : {:?}));
                        let Some(member_name) = accessor.child_by_field_name("name") else {
                            return Err(DscError::Parser(t!("parser.expression.memberNotFound").to_string()));
                        };
                        let member = member_name.utf8_text(statement_bytes)?;
                        Accessor::Member(member.to_string())
                    },
                    "index" => {
                        debug!("{}", t!("parser.expression.parsingIndexAccessor", index = node_to_string(&accessor, statement_bytes)? : {:?}));
                        let Some(index_value) = accessor.child_by_field_name("indexValue") else {
                            return Err(DscError::Parser(t!("parser.expression.indexNotFound").to_string()));
                        };
                        debug!("{}", t!("parser.expression.indexValue", value = node_to_string(&index_value, statement_bytes)? : {:?}, kind = index_value.kind()));
                        match index_value.kind() {
                            "number" => {
                                let value = index_value.utf8_text(statement_bytes)?;
                                let number: i64 = value.parse().map_err(|_| DscError::Parser(t!("parser.expression.indexNotValid").to_string()))?;
                                Accessor::Index(Value::Number(number.into()))
                            },
                            "propertyName" => {
                                let Some(string_node) = index_value.child_by_field_name("string") else {
                                    return Err(DscError::Parser(t!("parser.expression.propertyNameNotString").to_string()));
                                };
                                let value = string_node.utf8_text(statement_bytes)?;
                                debug!("{}", t!("parser.expression.propertyNameValue", value = value : {:?}));
                                Accessor::Index(Value::String(value.to_string()))
                            },
                            "expression" => {
                                let expression = Expression::new(statement_bytes, &index_value)?;
                                Accessor::IndexExpression(expression)
                            },
                            _ => {
                                return Err(DscError::Parser(t!("parser.expression.invalidIndexValueKind", kind = index_value.kind()).to_string()));
                            },
                        }
                    },
                    _ => {
                        return Err(DscError::Parser(t!("parser.expression.invalidAccessorKind", kind = accessor_kind).to_string()));
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
        let is_secure = is_secure_value(&result);
        if self.function.name() != "secret" && !is_secure {
            let result_json = serde_json::to_string(&result)?;
            trace!("{}", t!("parser.expression.functionResult", results = result_json));
        } else {
            trace!("{}", t!("parser.expression.functionResultSecure"));
        }
        if self.accessors.is_empty() {
            Ok(result)
        }
        else {
            debug!("{}", t!("parser.expression.evalAccessors"));
            let mut value = result;
            for accessor in &self.accessors {
                let mut index = Value::Null;
                match accessor {
                    Accessor::Member(member) => {
                        debug!("{}", t!("parser.expression.evaluatingMemberAccessor", name = member : {:?}));
                        if let Some(object) = value.as_object() {
                            if !object.contains_key(member) {
                                return Err(DscError::Parser(t!("parser.expression.memberNameNotFound", member = member).to_string()));
                            }
                            value = object[member].clone();
                        } else {
                            return Err(DscError::Parser(t!("parser.expression.accessOnNonObject").to_string()));
                        }
                    },
                    Accessor::Index(index_value) => {
                        debug!("{}", t!("parser.expression.evaluatingIndexAccessor", index = index_value : {:?}));
                        index = index_value.clone();
                    },
                    Accessor::IndexExpression(expression) => {
                        debug!("{}", t!("parser.expression.evaluatingIndexExpression", expression = expression : {:?}));
                        index = expression.invoke(function_dispatcher, context)?;
                        trace!("{}", t!("parser.expression.expressionResult", index = index : {:?}));
                    },
                }

                if index.is_number() {
                    if let Some(array) = value.as_array() {
                        let Some(index) = index.as_u64() else {
                            return Err(DscError::Parser(t!("parser.expression.indexNotValid").to_string()));
                        };
                        let index = usize::try_from(index)?;
                        if index >= array.len() {
                            return Err(DscError::Parser(t!("parser.expression.indexOutOfBounds").to_string()));
                        }
                        value = array[index].clone();
                    } else {
                        return Err(DscError::Parser(t!("parser.expression.indexOnNonArray").to_string()));
                    }
                }
                else if index.is_string() {
                    let index = index.as_str().ok_or_else(|| DscError::Parser(t!("parser.expression.indexNotValid").to_string()))?;
                    if let Some(object) = value.as_object() {
                        if !object.contains_key(index) {
                            return Err(DscError::Parser(t!("parser.expression.memberNameNotFound", member = index).to_string()));
                        }
                        value = object[index].clone();
                    } else {
                        return Err(DscError::Parser(t!("parser.expression.accessOnNonObject").to_string()));
                    }
                }
                else if !index.is_null() {
                    return Err(DscError::Parser(t!("parser.expression.invalidIndexType").to_string()));
                }
            }

            if is_secure {
                trace!("{}", t!("parser.expression.accessorResult", result = "<secureValue>"));
            } else {
                trace!("{}", t!("parser.expression.accessorResult", result = value : {:?}));
            }
            Ok(value)
        }
    }
}
