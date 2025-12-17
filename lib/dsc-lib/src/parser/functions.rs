// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use serde_json::{Number, Value};
use tracing::debug;
use tree_sitter::Node;

use crate::DscError;
use crate::configure::context::Context;
use crate::parser::{
    expressions::Expression,
    FunctionDispatcher,
};

#[derive(Clone, Debug)]
pub struct Function {
    name: String,
    args: Option<Vec<FunctionArg>>,
}

#[derive(Clone, Debug)]
pub enum FunctionArg {
    Value(Value),
    Expression(Expression),
    Lambda(Lambda),
}

/// Represents a lambda expression for use in DSC function expressions.
///
/// Lambda expressions are anonymous functions created using the `lambda()` function
/// and are primarily used with higher-order functions like `map()` to transform data.
/// Each lambda is stored in the context's lambda registry with a unique UUID identifier.
///
/// # Structure
///
/// A lambda consists of:
/// - **parameters**: A list of parameter names (e.g., `["item", "index"]`) that will be
///   bound to values when the lambda is invoked
/// - **body**: An expression tree that is evaluated with the bound parameters in scope
///
/// # Usage in DSC
///
/// Lambdas are created using the `lambda()` function syntax:
/// ```text
/// "[lambda(['item', 'index'], mul(variables('item'), 2))]"
/// ```
///
/// The lambda is stored in the context and referenced by UUID:
/// ```text
/// __lambda_<uuid>
/// ```
///
/// When used with `map()`, the lambda is invoked for each array element with bound parameters:
/// ```text
/// "[map(createArray(1, 2, 3), lambda(['item'], mul(variables('item'), 2)))]"
/// ```
///
/// # Lifetime
///
/// Lambdas are stored for the duration of a single configuration evaluation and are
/// automatically cleaned up when the `Context` is dropped at the end of processing.
/// Each configuration evaluation starts with a fresh, empty lambda registry.
#[derive(Clone, Debug)]
pub struct Lambda {
    pub parameters: Vec<String>,
    pub body: Expression,
}

impl Function {
    /// Create a new `Function` instance.
    ///
    /// # Arguments
    ///
    /// * `function_dispatcher` - The function dispatcher to use.
    /// * `statement` - The statement that the function is part of.
    /// * `function` - The function node.
    ///
    /// # Errors
    ///
    /// This function will return an error if the function node is not valid.
    pub fn new(statement_bytes: &[u8], function: &Node) -> Result<Self, DscError> {
        let mut function_name = None;
        let mut function_args = None;
        let mut cursor = function.walk();
        for member in function.named_children(&mut cursor) {
            match member.kind() {
                "arguments" => function_args = Some(member),
                "functionName" => function_name = Some(member),
                "ERROR" => return Err(DscError::Parser(t!("parser.functions.foundErrorNode").to_string())),
                _ => {}
            }
        }
        let Some(name) = function_name else {
            return Err(DscError::Parser(t!("parser.functions.nameNodeNotFound").to_string()));
        };
        let args = convert_args_node(statement_bytes, function_args.as_ref())?;
        let name = name.utf8_text(statement_bytes)?;
        debug!("{}", t!("parser.functions.functionName", name = name));
        Ok(Function{
            name: name.to_string(),
            args})
    }

    /// Invoke the function.
    ///
    /// # Errors
    ///
    /// This function will return an error if the function fails to execute.
    pub fn invoke(&self, function_dispatcher: &FunctionDispatcher, context: &Context) -> Result<Value, DscError> {
        // Special handling for lambda() function - pass raw args through context
        if self.name.to_lowercase() == "lambda" {
            // Store raw args in context for lambda function to access
            *context.lambda_raw_args.borrow_mut() = self.args.clone();
            let result = function_dispatcher.invoke("lambda", &[], context);
            // Clear raw args
            *context.lambda_raw_args.borrow_mut() = None;
            return result;
        }

        // if any args are expressions, we need to invoke those first
        let mut resolved_args: Vec<Value> = vec![];
        if let Some(args) = &self.args {
            for arg in args {
                match arg {
                    FunctionArg::Expression(expression) => {
                        debug!("{}", t!("parser.functions.argIsExpression"));
                        let value = expression.invoke(function_dispatcher, context)?;
                        resolved_args.push(value.clone());
                    },
                    FunctionArg::Value(value) => {
                        debug!("{}", t!("parser.functions.argIsValue", value = value : {:?}));
                        resolved_args.push(value.clone());
                    },
                    FunctionArg::Lambda(_lambda) => {
                        return Err(DscError::Parser(t!("parser.functions.unexpectedLambda").to_string()));
                    }
                }
            }
        }

        function_dispatcher.invoke(&self.name, &resolved_args, context)
    }

    /// Get the name of the function.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

fn convert_args_node(statement_bytes: &[u8], args: Option<&Node>) -> Result<Option<Vec<FunctionArg>>, DscError> {
    let Some(args) = args else {
        return Ok(None);
    };
    let mut result = vec![];
    let mut cursor = args.walk();
    for arg in args.named_children(&mut cursor) {
        match arg.kind() {
            "string" => {
                let value = arg.utf8_text(statement_bytes)?;
                // Resolve escaped single quotes
                result.push(FunctionArg::Value(Value::String(value.to_string().replace("''", "'"))));
            },
            "number" => {
                let value = arg.utf8_text(statement_bytes)?;
                result.push(FunctionArg::Value(Value::Number(Number::from(value.parse::<i32>()?))));
            },
            "boolean" => {
                let value = arg.utf8_text(statement_bytes)?;
                result.push(FunctionArg::Value(Value::Bool(value.parse::<bool>()?)));
            },
            "expression" => {
                // TODO: this is recursive, we may want to stop at a specific depth
                let expression = Expression::new(statement_bytes, &arg)?;
                result.push(FunctionArg::Expression(expression));
            },
            _ => {
                return Err(DscError::Parser(t!("parser.functions.unknownArgType", kind = arg.kind()).to_string()));
            }
        }
    }
    Ok(Some(result))
}
