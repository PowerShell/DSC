// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    configure::parameters::SecureString,
    dscerror::DscError,
    dscresources::{
        command_resource::invoke_command,
    },
    extensions::{
        dscextension::{
            Capability,
            DscExtension,
        },
        extension_manifest::ExtensionManifest,
    },
    schemas::dsc_repo::DscRepoSchema
};

use rust_i18n::t;
use schemars::{JsonSchema, Schema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, warn};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
#[schemars(inline)]
pub enum SecretArgKind {
    /// A static string argument to pass to the command, like `get` or `--quiet`.
    String(String),
    /// The argument that accepts the secret name. DSC passes the name of the secret to retrieve
    /// after this argument. The arguments must define this argument exactly once.
    Name {
        /// The argument that accepts the secret name, like `--name` or `--secret-name`.
        #[serde(rename = "nameArg")]
        name_arg: String,
    },
    /// The argument that accepts the vault name. DSC passes the name of the vault after this
    /// argument when the `secret()` function specifies a vault. The arguments may define this
    /// argument at most once.
    Vault {
        /// The argument that accepts the vault name, like `--vault` or `--vault-name`.
        #[serde(rename = "vaultArg")]
        vault_arg: String,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[schemars(
    transform = SecretMethod::transform_export_schema_uris,
    transform = SecretMethod::transform_schema_docs,
    transform = SecretMethod::transform_args_constraints
)]
#[dsc_repo_schema(base_name = "manifest.secret", folder_path = "extension")]
pub struct SecretMethod {
    /// The command to run to retrieve a secret.
    pub executable: String,
    /// The arguments to pass to the command to retrieve a secret. The arguments must define the
    /// secret name input argument exactly once and may define the vault input argument at most
    /// once.
    pub args: Vec<SecretArgKind>,
}

impl SecretMethod {
    /// Validates that the arguments define the secret name input argument exactly once and the
    /// vault input argument at most once.
    ///
    /// Without the secret name input argument, DSC can't tell the extension which secret to
    /// retrieve, so the extension can't function as a secret provider.
    ///
    /// # Errors
    ///
    /// Returns [`DscError::InvalidManifest`] when the arguments don't define the secret name input
    /// argument, define it more than once, or define the vault input argument more than once.
    pub fn validate_args(&self) -> Result<(), DscError> {
        let name_arg_count = self.args.iter().filter(|arg| matches!(arg, SecretArgKind::Name { .. })).count();
        let vault_arg_count = self.args.iter().filter(|arg| matches!(arg, SecretArgKind::Vault { .. })).count();

        if name_arg_count == 0 {
            return Err(DscError::InvalidManifest(t!("extensions.secret.missingNameArg").to_string()));
        }
        if name_arg_count > 1 {
            return Err(DscError::InvalidManifest(t!("extensions.secret.multipleNameArgs", count = name_arg_count).to_string()));
        }
        if vault_arg_count > 1 {
            return Err(DscError::InvalidManifest(t!("extensions.secret.multipleVaultArgs", count = vault_arg_count).to_string()));
        }

        Ok(())
    }

    /// Adds the validation subschemas that require `args` to define the secret name input
    /// argument exactly once and the vault input argument at most once.
    ///
    /// The subschemas are defined separately so that each failure reports a specific message.
    /// The `errorMessage` keyword is only emitted in the VS Code form of the schema.
    fn transform_args_constraints(schema: &mut Schema) {
        let docs_url = "https://learn.microsoft.com/powershell/dsc/reference/schemas/extension/manifest/secret";
        schema.insert("allOf".to_string(), json!([
            {
                "title": "Missing secret name input argument",
                "properties": {
                    "args": {
                        "errorMessage": format!(
                            "The `secret` command doesn't define the secret name input argument. If you don't define the secret name input argument, DSC can't pass the secret name to the extension for retrieval. You must define exactly one argument in `secret.args` as a JSON object with the `nameArg` property. For more information, see: {docs_url}"
                        ),
                        "contains": { "type": "object", "required": ["nameArg"] },
                        "minContains": 1
                    }
                }
            },
            {
                "title": "Multiple secret name input arguments",
                "properties": {
                    "args": {
                        "errorMessage": format!(
                            "The `secret` command defines the secret name input argument more than once. You must define exactly one argument in `secret.args` as a JSON object with the `nameArg` property and remove the additional secret name input arguments. For more information, see: {docs_url}"
                        ),
                        "contains": { "type": "object", "required": ["nameArg"] },
                        "maxContains": 1
                    }
                }
            },
            {
                "title": "Multiple vault input arguments",
                "properties": {
                    "args": {
                        "errorMessage": format!(
                            "The `secret` command defines the vault input argument more than once. You can define at most one argument in `secret.args` as a JSON object with the `vaultArg` property. For more information, see: {docs_url}"
                        ),
                        "contains": { "type": "object", "required": ["vaultArg"] },
                        "minContains": 0,
                        "maxContains": 1
                    }
                }
            }
        ]));
    }
}

impl DscExtension {
    /// Retrieve a secret using the extension.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the secret to retrieve.
    /// * `vault` - An optional vault name to use for the secret.
    ///
    /// # Returns
    ///
    /// A result containing the secret as a string or an error.
    ///
    /// # Errors
    ///
    /// This function will return an error if the secret retrieval fails or if the extension does not support the secret capability.
    pub fn secret(&self, name: &str, vault: Option<&str>) -> Result<Option<String>, DscError> {
        if self.capabilities.contains(&Capability::Secret) {
            debug!("{}", t!("extensions.dscextension.retrievingSecretFromExtension", name = name, extension = self.type_name));
            let extension = match serde_json::from_value::<ExtensionManifest>(self.manifest.clone()) {
                Ok(manifest) => manifest,
                Err(err) => {
                    return Err(DscError::Manifest(self.type_name.to_string(), err));
                }
            };
            let Some(secret) = extension.secret else {
                return Err(DscError::UnsupportedCapability(self.type_name.to_string(), Capability::Secret.to_string()));
            };
            secret.validate_args()?;
            let args = process_secret_args(&secret.args, name, vault);
            if let Some(deprecation_message) = extension.deprecation_message.as_ref() {
                warn!("{}", t!("extensions.dscextension.deprecationMessage", extension = self.type_name, message = deprecation_message));
            }
            let (_exit_code, stdout, _stderr) = invoke_command(
                &secret.executable,
                Some(args),
                vault,
                Some(&self.directory),
                None,
                extension.exit_codes.as_ref(),
            )?;
            if stdout.is_empty() {
                debug!("{}", t!("extensions.dscextension.extensionReturnedNoSecret", extension = self.type_name));
                Ok(None)
            } else {
                // see if multiple lines were returned
                let secret = if stdout.lines().count() > 1 {
                    return Err(DscError::NotSupported(t!("extensions.dscextension.secretMultipleLinesReturned", extension = self.type_name).to_string()));
                } else {
                    debug!("{}", t!("extensions.dscextension.extensionReturnedSecret", extension = self.type_name));
                    // remove any trailing newline characters
                    stdout.trim_end_matches('\n').to_string()
                };
                let secure_string = SecureString {
                    secure_string: secret.clone(),
                };
                Ok(Some(serde_json::to_string(&secure_string)?))
            }
        } else {
            Err(DscError::UnsupportedCapability(
                self.type_name.to_string(),
                Capability::Secret.to_string()
            ))
        }
    }
}

fn process_secret_args(args: &[SecretArgKind], name: &str, vault: Option<&str>) -> Vec<String> {
    let mut processed_args = Vec::<String>::new();
    for arg in args {
        match arg {
            SecretArgKind::String(s) => {
                processed_args.push(s.clone());
            },
            SecretArgKind::Name { name_arg } => {
                processed_args.push(name_arg.clone());
                processed_args.push(name.to_string());
            },
            SecretArgKind::Vault { vault_arg } => {
                if let Some(value) = vault {
                    processed_args.push(vault_arg.clone());
                    processed_args.push(value.to_string());
                }
            },
        }
    }

    processed_args
}

#[cfg(test)]
mod test {
    use super::{SecretArgKind, SecretMethod, process_secret_args};
    use crate::dscerror::DscError;
    use crate::extensions::extension_manifest::ExtensionManifest;
    use crate::schemas::dsc_repo::DscRepoSchema;
    use schemars::schema_for;
    use serde_json::{Value, json};

    fn name_arg() -> SecretArgKind {
        SecretArgKind::Name { name_arg: "--name".to_string() }
    }

    fn vault_arg() -> SecretArgKind {
        SecretArgKind::Vault { vault_arg: "--vault".to_string() }
    }

    fn secret_method(args: Vec<SecretArgKind>) -> SecretMethod {
        SecretMethod { executable: "secret".to_string(), args }
    }

    #[test]
    fn validate_args_accepts_single_name_arg() {
        let method = secret_method(vec![SecretArgKind::String("get".to_string()), name_arg()]);
        assert!(method.validate_args().is_ok());
    }

    #[test]
    fn validate_args_accepts_single_name_and_vault_arg() {
        let method = secret_method(vec![vault_arg(), name_arg()]);
        assert!(method.validate_args().is_ok());
    }

    #[test]
    fn validate_args_rejects_missing_name_arg() {
        let method = secret_method(vec![SecretArgKind::String("get".to_string()), vault_arg()]);
        let err = method.validate_args().unwrap_err();
        assert!(matches!(err, DscError::InvalidManifest(_)));
        assert!(err.to_string().contains("doesn't define the secret name input argument"), "{err}");
    }

    #[test]
    fn validate_args_rejects_empty_args() {
        let method = secret_method(vec![]);
        assert!(method.validate_args().is_err());
    }

    #[test]
    fn validate_args_rejects_multiple_name_args() {
        let method = secret_method(vec![name_arg(), name_arg()]);
        let err = method.validate_args().unwrap_err();
        assert!(matches!(err, DscError::InvalidManifest(_)));
        assert!(err.to_string().contains("defines the secret name input argument 2 times"), "{err}");
    }

    #[test]
    fn validate_args_rejects_multiple_vault_args() {
        let method = secret_method(vec![name_arg(), vault_arg(), vault_arg()]);
        let err = method.validate_args().unwrap_err();
        assert!(matches!(err, DscError::InvalidManifest(_)));
        assert!(err.to_string().contains("defines the vault input argument 2 times"), "{err}");
    }

    #[test]
    fn process_secret_args_preserves_order_and_inserts_values() {
        let args = vec![SecretArgKind::String("get".to_string()), vault_arg(), name_arg(), SecretArgKind::String("--quiet".to_string())];
        let processed = process_secret_args(&args, "apiToken", Some("services"));
        assert_eq!(processed, vec!["get", "--vault", "services", "--name", "apiToken", "--quiet"]);
    }

    #[test]
    fn process_secret_args_omits_vault_arg_without_vault() {
        let args = vec![SecretArgKind::String("get".to_string()), name_arg(), vault_arg()];
        let processed = process_secret_args(&args, "apiToken", None);
        assert_eq!(processed, vec!["get", "--name", "apiToken"]);
    }

    #[test]
    fn manifest_without_secret_args_fails_to_deserialize() {
        let manifest = json!({
            "$schema": ExtensionManifest::default_schema_id_uri(),
            "type": "Test/Secret",
            "version": "0.1.0",
            "secret": { "executable": "secret" }
        });
        let err = serde_json::from_value::<ExtensionManifest>(manifest).unwrap_err();
        assert!(err.to_string().contains("missing field `args`"), "{err}");
    }

    #[test]
    fn manifest_with_secret_args_deserializes() {
        let manifest = json!({
            "$schema": ExtensionManifest::default_schema_id_uri(),
            "type": "Test/Secret",
            "version": "0.1.0",
            "secret": {
                "executable": "secret",
                "args": ["get", { "nameArg": "--name" }, { "vaultArg": "--vault" }]
            }
        });
        let manifest = serde_json::from_value::<ExtensionManifest>(manifest).unwrap();
        let secret = manifest.secret.unwrap();
        assert_eq!(secret.args, vec![SecretArgKind::String("get".to_string()), name_arg(), vault_arg()]);
        assert!(secret.validate_args().is_ok());
    }

    #[test]
    fn schema_requires_args_and_constrains_input_arguments() {
        let schema = schema_for!(SecretMethod).to_value();
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&Value::String("executable".to_string())), "{schema}");
        assert!(required.contains(&Value::String("args".to_string())), "{schema}");

        let constraints = schema["allOf"].as_array().unwrap();
        assert_eq!(constraints.len(), 3, "{schema}");
        assert_eq!(constraints[0]["properties"]["args"]["minContains"], json!(1));
        assert_eq!(constraints[0]["properties"]["args"]["contains"]["required"], json!(["nameArg"]));
        assert_eq!(constraints[1]["properties"]["args"]["maxContains"], json!(1));
        assert_eq!(constraints[1]["properties"]["args"]["contains"]["required"], json!(["nameArg"]));
        assert_eq!(constraints[2]["properties"]["args"]["maxContains"], json!(1));
        assert_eq!(constraints[2]["properties"]["args"]["contains"]["required"], json!(["vaultArg"]));
    }

    #[test]
    fn schema_rejects_args_without_name_arg() {
        let schema = schema_for!(SecretMethod).to_value();
        let validator = jsonschema::validator_for(&schema).unwrap();

        assert!(validator.is_valid(&json!({ "executable": "secret", "args": ["get", { "nameArg": "--name" }] })));
        assert!(validator.is_valid(&json!({ "executable": "secret", "args": [{ "vaultArg": "--vault" }, { "nameArg": "--name" }] })));
        assert!(!validator.is_valid(&json!({ "executable": "secret" })));
        assert!(!validator.is_valid(&json!({ "executable": "secret", "args": ["get"] })));
        assert!(!validator.is_valid(&json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "nameArg": "--secret" }] })));
        assert!(!validator.is_valid(&json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "vaultArg": "--vault" }, { "vaultArg": "--store" }] })));
    }
}
