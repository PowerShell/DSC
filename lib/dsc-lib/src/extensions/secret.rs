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
    schemas::dsc_repo::{DscRepoSchema, schema_i18n}
};

use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::Deref;
use tracing::{debug, warn};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
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

/// Defines the arguments to pass to the `secret` command of an extension.
///
/// The arguments must define the secret name input argument exactly once and may define the vault
/// input argument at most once. Without the secret name input argument, DSC can't tell the
/// extension which secret to retrieve, so the extension can't function as a secret provider.
///
/// Deserializing arguments that break these rules fails, so a deserialized instance is always
/// valid and DSC doesn't need to validate the arguments again before invoking the extension.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(try_from = "Vec<Value>")]
#[schemars(inline, !try_from)]
pub struct SecretArgs(Vec<SecretArgKind>);

impl SecretArgs {
    /// Creates the arguments for the `secret` command from a list of argument definitions.
    ///
    /// # Arguments
    ///
    /// * `args` - The argument definitions in the order to pass them to the command.
    ///
    /// # Errors
    ///
    /// Returns [`DscError::InvalidManifest`] when the arguments don't define the secret name input
    /// argument, define it more than once, or define the vault input argument more than once.
    pub fn new(args: Vec<SecretArgKind>) -> Result<Self, DscError> {
        Self::validate(&args)?;
        Ok(Self(args))
    }

    /// Creates the arguments for the `secret` command from the JSON values of a manifest.
    ///
    /// Each value must be a string, an object with only the `nameArg` property, or an object with
    /// only the `vaultArg` property.
    ///
    /// # Arguments
    ///
    /// * `values` - The JSON values of the `args` property in the order to pass them to the command.
    ///
    /// # Errors
    ///
    /// Returns [`DscError::InvalidManifest`] when a value isn't a valid argument definition or when
    /// the arguments break the rules described for [`SecretArgs::new`].
    pub fn from_values(values: Vec<Value>) -> Result<Self, DscError> {
        let mut args = Vec::with_capacity(values.len());
        for (index, value) in values.iter().enumerate() {
            match SecretArgKind::deserialize(value) {
                Ok(arg) => args.push(arg),
                Err(_) => {
                    return Err(DscError::InvalidManifest(t!("extensions.secret.invalidArg", position = index + 1, arg = value).to_string()));
                }
            }
        }
        Self::new(args)
    }

    /// Validates that the arguments define the secret name input argument exactly once and the
    /// vault input argument at most once.
    fn validate(args: &[SecretArgKind]) -> Result<(), DscError> {
        let name_arg_count = args.iter().filter(|arg| matches!(arg, SecretArgKind::Name { .. })).count();
        let vault_arg_count = args.iter().filter(|arg| matches!(arg, SecretArgKind::Vault { .. })).count();

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
}

impl TryFrom<Vec<Value>> for SecretArgs {
    type Error = DscError;

    fn try_from(values: Vec<Value>) -> Result<Self, Self::Error> {
        Self::from_values(values)
    }
}

impl TryFrom<Vec<SecretArgKind>> for SecretArgs {
    type Error = DscError;

    fn try_from(args: Vec<SecretArgKind>) -> Result<Self, Self::Error> {
        Self::new(args)
    }
}

impl Deref for SecretArgs {
    type Target = [SecretArgKind];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema, DscRepoSchema)]
#[schemars(
    transform = SecretMethod::transform_export_schema_uris,
    transform = SecretMethod::transform_schema_docs
)]
#[dsc_repo_schema(base_name = "manifest.secret", folder_path = "extension")]
pub struct SecretMethod {
    /// The command to run to retrieve a secret.
    pub executable: String,
    /// The arguments to pass to the command to retrieve a secret. The arguments must define the
    /// secret name input argument exactly once and may define the vault input argument at most
    /// once.
    #[schemars(extend("allOf" = [
        {
            "title": schema_i18n!("args.constraints.nameArg.title"),
            "description": schema_i18n!("args.constraints.nameArg.description"),
            "markdownDescription": schema_i18n!("args.constraints.nameArg.markdownDescription"),
            "errorMessage": schema_i18n!("args.constraints.nameArg.errorMessage"),
            "contains": { "type": "object", "required": ["nameArg"] },
            "minContains": 1,
            "maxContains": 1,
        },
        {
            "title": schema_i18n!("args.constraints.vaultArg.title"),
            "description": schema_i18n!("args.constraints.vaultArg.description"),
            "markdownDescription": schema_i18n!("args.constraints.vaultArg.markdownDescription"),
            "errorMessage": schema_i18n!("args.constraints.vaultArg.errorMessage"),
            "contains": { "type": "object", "required": ["vaultArg"] },
            "minContains": 0,
            "maxContains": 1,
        },
    ]))]
    pub args: SecretArgs,
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
    use super::{SecretArgKind, SecretArgs, SecretMethod, process_secret_args};
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

    fn manifest_with_secret(secret: Value) -> Value {
        json!({
            "$schema": ExtensionManifest::default_schema_id_uri(),
            "type": "Test/Secret",
            "version": "0.1.0",
            "secret": secret
        })
    }

    #[test]
    fn new_accepts_single_name_arg() {
        let args = SecretArgs::new(vec![SecretArgKind::String("get".to_string()), name_arg()]).unwrap();
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn new_accepts_single_name_and_vault_arg() {
        assert!(SecretArgs::new(vec![vault_arg(), name_arg()]).is_ok());
    }

    #[test]
    fn new_rejects_missing_name_arg() {
        let err = SecretArgs::new(vec![SecretArgKind::String("get".to_string()), vault_arg()]).unwrap_err();
        assert!(matches!(err, DscError::InvalidManifest(_)));
        assert!(err.to_string().contains("doesn't define the secret name input argument"), "{err}");
    }

    #[test]
    fn new_rejects_empty_args() {
        assert!(SecretArgs::new(vec![]).is_err());
    }

    #[test]
    fn new_rejects_multiple_name_args() {
        let err = SecretArgs::new(vec![name_arg(), name_arg()]).unwrap_err();
        assert!(matches!(err, DscError::InvalidManifest(_)));
        assert!(err.to_string().contains("defines the secret name input argument 2 times"), "{err}");
    }

    #[test]
    fn new_rejects_multiple_vault_args() {
        let err = SecretArgs::new(vec![name_arg(), vault_arg(), vault_arg()]).unwrap_err();
        assert!(matches!(err, DscError::InvalidManifest(_)));
        assert!(err.to_string().contains("defines the vault input argument 2 times"), "{err}");
    }

    #[test]
    fn from_values_converts_each_argument_kind() {
        let args = SecretArgs::from_values(vec![json!("get"), json!({ "nameArg": "--name" }), json!({ "vaultArg": "--vault" })]).unwrap();
        assert_eq!(*args, [SecretArgKind::String("get".to_string()), name_arg(), vault_arg()]);
    }

    #[test]
    fn from_values_rejects_object_with_name_and_vault_arg() {
        let err = SecretArgs::from_values(vec![json!("get"), json!({ "nameArg": "--name", "vaultArg": "--vault" })]).unwrap_err();
        assert!(matches!(err, DscError::InvalidManifest(_)));
        assert!(err.to_string().contains("Argument 2 of the 'secret' command isn't valid"), "{err}");
        assert!(err.to_string().contains(r#"{"nameArg":"--name","vaultArg":"--vault"}"#), "{err}");
    }

    #[test]
    fn from_values_rejects_object_with_unknown_property() {
        let err = SecretArgs::from_values(vec![json!({ "nameArg": "--name", "other": true })]).unwrap_err();
        assert!(err.to_string().contains("Argument 1 of the 'secret' command isn't valid"), "{err}");
    }

    #[test]
    fn from_values_rejects_non_string_scalar() {
        let err = SecretArgs::from_values(vec![json!(5), json!({ "nameArg": "--name" })]).unwrap_err();
        assert!(err.to_string().contains("Argument 1 of the 'secret' command isn't valid"), "{err}");
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
        let manifest = manifest_with_secret(json!({ "executable": "secret" }));
        let err = serde_json::from_value::<ExtensionManifest>(manifest).unwrap_err();
        assert!(err.to_string().contains("missing field `args`"), "{err}");
    }

    #[test]
    fn manifest_without_name_arg_fails_to_deserialize() {
        let manifest = manifest_with_secret(json!({ "executable": "secret", "args": ["get"] }));
        let err = serde_json::from_value::<ExtensionManifest>(manifest).unwrap_err();
        assert!(err.to_string().contains("doesn't define the secret name input argument"), "{err}");
    }

    #[test]
    fn manifest_with_multiple_name_args_fails_to_deserialize() {
        let manifest = manifest_with_secret(json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "nameArg": "--secret" }] }));
        let err = serde_json::from_value::<ExtensionManifest>(manifest).unwrap_err();
        assert!(err.to_string().contains("defines the secret name input argument 2 times"), "{err}");
    }

    #[test]
    fn manifest_with_ambiguous_arg_fails_to_deserialize() {
        let manifest = manifest_with_secret(json!({ "executable": "secret", "args": [{ "nameArg": "--name", "vaultArg": "--vault" }] }));
        let err = serde_json::from_value::<ExtensionManifest>(manifest).unwrap_err();
        assert!(err.to_string().contains("Argument 1 of the 'secret' command isn't valid"), "{err}");
    }

    #[test]
    fn manifest_with_secret_args_round_trips() {
        let args = json!(["get", { "nameArg": "--name" }, { "vaultArg": "--vault" }]);
        let manifest = manifest_with_secret(json!({ "executable": "secret", "args": args }));
        let manifest = serde_json::from_value::<ExtensionManifest>(manifest).unwrap();
        let secret = manifest.secret.unwrap();
        assert_eq!(*secret.args, [SecretArgKind::String("get".to_string()), name_arg(), vault_arg()]);
        assert_eq!(serde_json::to_value(&secret).unwrap()["args"], args);
    }

    #[test]
    fn schema_requires_args_and_constrains_input_arguments() {
        let schema = schema_for!(SecretMethod).to_value();
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&Value::String("executable".to_string())), "{schema}");
        assert!(required.contains(&Value::String("args".to_string())), "{schema}");

        let args = &schema["properties"]["args"];
        assert_eq!(args["type"], json!("array"), "{schema}");
        let constraints = args["allOf"].as_array().unwrap();
        assert_eq!(constraints.len(), 2, "{schema}");
        assert_eq!(constraints[0]["contains"]["required"], json!(["nameArg"]));
        assert_eq!(constraints[0]["minContains"], json!(1));
        assert_eq!(constraints[0]["maxContains"], json!(1));
        assert_eq!(constraints[1]["contains"]["required"], json!(["vaultArg"]));
        assert_eq!(constraints[1]["minContains"], json!(0));
        assert_eq!(constraints[1]["maxContains"], json!(1));
        for constraint in constraints {
            for keyword in ["title", "description", "markdownDescription", "errorMessage"] {
                assert!(constraint[keyword].is_string(), "{keyword} missing: {constraint}");
            }
        }
    }

    #[test]
    fn schema_validates_input_argument_rules() {
        let schema = schema_for!(SecretMethod).to_value();
        let validator = jsonschema::validator_for(&schema).unwrap();

        assert!(validator.is_valid(&json!({ "executable": "secret", "args": ["get", { "nameArg": "--name" }] })));
        assert!(validator.is_valid(&json!({ "executable": "secret", "args": [{ "vaultArg": "--vault" }, { "nameArg": "--name" }] })));
        assert!(!validator.is_valid(&json!({ "executable": "secret" })));
        assert!(!validator.is_valid(&json!({ "executable": "secret", "args": ["get"] })));
        assert!(!validator.is_valid(&json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "nameArg": "--secret" }] })));
        assert!(!validator.is_valid(&json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "vaultArg": "--vault" }, { "vaultArg": "--store" }] })));
        assert!(!validator.is_valid(&json!({ "executable": "secret", "args": [{ "nameArg": "--name", "vaultArg": "--vault" }] })));
    }
}
