// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct SystemRoot {}

/// Implements the `systemRoot` function.
/// This function returns the value of the specified system root path.
impl Function for SystemRoot {
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

        Ok(Value::String(context.system_root.to_string_lossy().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use std::path::PathBuf;

    #[test]
    fn init() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[systemRoot()]", &Context::new()).unwrap();
        // on windows, the default is SYSTEMDRIVE env var
        #[cfg(target_os = "windows")]
        assert_eq!(result, format!("{}\\", std::env::var("SYSTEMDRIVE").unwrap()));
        // on linux/macOS, the default is /
        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, "/");
    }

    #[test]
    fn simple() {
        let mut parser = Statement::new().unwrap();
        let mut context = Context::new();
        let separator = std::path::MAIN_SEPARATOR;
        context.system_root = PathBuf::from(format!("{separator}mnt"));
        let result = parser.parse_and_execute("[systemRoot()]", &context).unwrap();
        assert_eq!(result, format!("{separator}mnt"));
    }

}
