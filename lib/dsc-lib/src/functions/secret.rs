// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::extensions::secret::resolve_secret;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct Secret {}

impl Function for Secret {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "secret".to_string(),
            description: t!("functions.secret.description").to_string(),
            syntax: t!("functions.secret.syntax").to_string(),
            constraints: Some(t!("functions.secret.constraints").to_string()),
            category: vec![FunctionCategory::Deployment],
            min_args: 1,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], context: &Context) -> Result<Value, DscError> {
        let secret_name = args[0].as_str().ok_or_else(|| {
            DscError::Function("secret".to_string(), t!("functions.secret.notString").to_string())
        })?.to_string();
        let vault_name: Option<String> = if args.len() > 1 {
            args[1].as_str().map(std::string::ToString::to_string)
        } else {
            None
        };

        resolve_secret(&context.extensions, &secret_name, vault_name.as_deref(), context, None, false)
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn not_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[secret(1)]", &Context::new());
        assert!(result.is_err());
    }
}
