// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct SystemRoot {}

/// Implements the `systemRoot` function.
/// This function returns the value of the specified system root path.
impl Function for SystemRoot {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "systemRoot".to_string(),
            description: t!("functions.systemRoot.description").to_string(),
            category: vec![FunctionCategory::System],
            min_args: 0,
            max_args: 0,
            accepted_arg_ordered_types: vec![],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, _args: &[Value], context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.systemRoot.invoked"));

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
