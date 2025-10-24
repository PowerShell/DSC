// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;
use which::which;

#[derive(Debug, Default)]
pub struct TryWhich {}

impl Function for TryWhich {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "tryWhich".to_string(),
            description: t!("functions.tryWhich.description").to_string(),
            category: vec![FunctionCategory::System],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![
                FunctionArgKind::String,
                FunctionArgKind::Null,
            ],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.tryWhich.invoked"));

        let exe = args[0].as_str().unwrap();
        match which(exe) {
            Ok(found_path) => {
                let path_str = found_path.to_string_lossy().to_string();
                Ok(Value::String(path_str))
            },
            Err(_) => {
                // In tryWhich, we return null if not found
                Ok(Value::Null)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::Value;

    #[test]
    fn exe_exists() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryWhich('dsc')]", &Context::new()).unwrap();
        #[cfg(windows)]
        assert!(result.as_str().unwrap().to_lowercase().ends_with("\\dsc.exe"));
        #[cfg(not(windows))]
        assert!(result.as_str().unwrap().ends_with("/dsc"));
    }

    #[test]
    fn invalid_exe() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[tryWhich('does_not_exist')]", &Context::new()).unwrap();
        assert_eq!(result, Value::Null);
    }
}
