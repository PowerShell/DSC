// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::extensions::dscextension::Capability;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;
use tracing::warn;

#[derive(Debug, Default)]
pub struct Secret {}

impl Function for Secret {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "secret".to_string(),
            description: t!("functions.secret.description").to_string(),
            category: FunctionCategory::System,
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

        // we query all extensions supporting the secret method to see if any of them can provide the secret.
        // if none can or if multiple provide different values, we return an error.
        let extensions = context.extensions.iter()
            .filter(|ext| ext.capabilities.contains(&Capability::Secret))
            .collect::<Vec<_>>();
        let mut secret_returned = false;
        let mut result: String = String::new();
        if extensions.is_empty() {
            return Err(DscError::Function("secret".to_string(), t!("functions.secret.noExtensions").to_string()));
        }
        for extension in extensions {
            match extension.secret(&secret_name, vault_name.as_deref()) {
                Ok(secret_result) => {
                    if let Some(secret_value) = secret_result {
                        if secret_returned && result != secret_value {
                            return Err(DscError::Function("secret".to_string(), t!("functions.secret.multipleSecrets", name = secret_name.clone()).to_string()));
                        }

                        result = secret_value;
                        secret_returned = true;
                    }
                },
                Err(err) => {
                    warn!("{}", t!("functions.secret.extensionReturnedError", extension = extension.type_name.clone(), error = err));
                }
            }
        }
        if secret_returned {
            Ok(Value::String(result))
        } else {
            Err(DscError::Function("secret".to_string(), t!("functions.secret.secretNotFound", name = secret_name).to_string()))
        }
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
