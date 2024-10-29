// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct MountedPath {}

/// Implements the 'mountedpath' function.
/// This function returns the value of the mounted path.
/// The optional parameter is a path appended to the mounted path.
/// Path is not validated as it might be used for creation.
impl Function for MountedPath {
    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> usize {
        1
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("Executing mountedpath function with args: {:?}", args);

        if args.len() == 1 {
            let path = args[0].as_str().ok_or(DscError::Parser("Invalid argument".to_string()))?;
            Ok(Value::String(format!("{}{}{path}", context.mounted_path, std::path::MAIN_SEPARATOR)))
        } else {
            Ok(Value::String(context.mounted_path.clone()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn no_arg() {
        let mut parser = Statement::new().unwrap();
        let mut context = Context::new();
        let separator = std::path::MAIN_SEPARATOR;
        context.mounted_path = format!("{separator}mnt");
        let result = parser.parse_and_execute("[mountedpath()]", &context).unwrap();
        assert_eq!(result, format!("{separator}mnt"));
    }

    #[test]
    fn with_arg() {
        let mut parser = Statement::new().unwrap();
        let mut context = Context::new();
        let separator = std::path::MAIN_SEPARATOR;
        context.mounted_path = format!("{separator}mnt");
        let result = parser.parse_and_execute("[mountedpath('foo')]", &context).unwrap();
        assert_eq!(result, format!("{separator}mnt{separator}foo"));
    }
}
