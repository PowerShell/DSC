// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct TargetPath {}

/// Implements the `targetPath` function.
/// This function returns the value of the mounted path.
/// The optional parameter is a path appended to the mounted path.
/// Path is not validated as it might be used for creation.
impl Function for TargetPath {
    fn min_args(&self) -> usize {
        0
    }

    fn max_args(&self) -> usize {
        0
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, _args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("Executing targetPath function");

        Ok(Value::String(context.target_path.to_string_lossy().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use std::{env, path::PathBuf};

    #[test]
    fn init() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[targetPath()]", &Context::new()).unwrap();
        // on windows, the default is SYSTEMDRIVE env var
        #[cfg(target_os = "windows")]
        assert_eq!(result, env::var("SYSTEMDRIVE").unwrap());
        // on linux/macOS, the default is /
        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, "/");
    }

    #[test]
    fn simple() {
        let mut parser = Statement::new().unwrap();
        let mut context = Context::new();
        let separator = std::path::MAIN_SEPARATOR;
        context.target_path = PathBuf::from(format!("{separator}mnt"));
        let result = parser.parse_and_execute("[targetPath()]", &context).unwrap();
        assert_eq!(result, format!("{separator}mnt"));
    }

}
