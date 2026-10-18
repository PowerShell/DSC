// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::{
    configure::parameters::{SecureString, SecureObject},
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

use miette::Diagnostic;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
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

// Display the argument definition as it appears in a manifest, like `"get"` or
// `{"nameArg":"--name"}`, so messages show the definition the author wrote.
impl Display for SecretArgKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        f.write_str(&json)
    }
}

/// Defines the errors that can occur when creating a [`SecretArgs`] instance.
#[derive(Debug, Clone, PartialEq, Error, Diagnostic)]
pub enum SecretArgsError {
    /// Indicates that the arguments for the `secret` command are invalid for one or more reasons.
    ///
    /// The `errors` field lists every problem found in the arguments, like an argument definition
    /// that isn't valid or a missing secret name input argument, so the manifest author can review
    /// all of them at once instead of discovering them one at a time.
    #[error("{t}", t = t!(
        "extensions.secret.invalidArgs",
        err = errors.iter().map(ToString::to_string).collect::<Vec<String>>().join("; ")
    ))]
    InvalidArgs {
        /// The specific problems found in the arguments.
        #[related]
        errors: Vec<SecretArgsError>,
    },

    /// Indicates that an argument definition isn't a string, an object with only the `nameArg`
    /// property, or an object with only the `vaultArg` property.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidArgs`] error.
    ///
    /// [`InvalidArgs`]: Self::InvalidArgs
    #[error("{t}", t = t!("extensions.secret.invalidArg", position = position, arg = value))]
    InvalidArg {
        /// The one-based position of the invalid argument in the `args` array.
        position: usize,
        /// The JSON value of the invalid argument.
        value: Value,
    },

    /// Indicates that the arguments don't define the secret name input argument.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidArgs`] error.
    ///
    /// [`InvalidArgs`]: Self::InvalidArgs
    #[error("{t}", t = t!("extensions.secret.missingNameArg"))]
    MissingNameArg,

    /// Indicates that the arguments define the secret name input argument more than once.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidArgs`] error.
    ///
    /// [`InvalidArgs`]: Self::InvalidArgs
    #[error("{t}", t = t!("extensions.secret.multipleNameArgs", count = count))]
    MultipleNameArgs {
        /// The number of secret name input arguments the arguments define.
        count: usize,
    },

    /// Indicates that the arguments define the vault input argument more than once.
    ///
    /// Validation raises this error in the `errors` field of the main [`InvalidArgs`] error.
    ///
    /// [`InvalidArgs`]: Self::InvalidArgs
    #[error("{t}", t = t!("extensions.secret.multipleVaultArgs", count = count))]
    MultipleVaultArgs {
        /// The number of vault input arguments the arguments define.
        count: usize,
    },
}

/// Defines the arguments to pass to the `secret` command of an extension.
///
/// The arguments must define the secret name input argument exactly once and may define the vault
/// input argument at most once. Without the secret name input argument, DSC can't tell the
/// extension which secret to retrieve, so the extension can't function as a secret provider.
///
/// Every constructor validates these rules, and deserializing arguments that break them fails, so
/// an instance is always valid and DSC doesn't need to validate the arguments again before
/// invoking the extension. The type implements [`AsRef`], [`Deref`], and [`IntoIterator`] to
/// enable ergonomically using an instance as the underlying vector, but not [`DerefMut`], because
/// mutating the arguments could break the rules.
///
/// [`DerefMut`]: std::ops::DerefMut
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema)]
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
    /// Returns [`SecretArgsError::InvalidArgs`] when the arguments break any of the rules. The
    /// error lists every rule the arguments break:
    ///
    /// * [`SecretArgsError::MissingNameArg`] when the arguments don't define the secret name input
    ///   argument.
    /// * [`SecretArgsError::MultipleNameArgs`] when the arguments define the secret name input
    ///   argument more than once.
    /// * [`SecretArgsError::MultipleVaultArgs`] when the arguments define the vault input argument
    ///   more than once.
    pub fn from_args(args: Vec<SecretArgKind>) -> Result<Self, SecretArgsError> {
        let mut errors = Vec::new();
        Self::validate(&args, &mut errors);
        Self::finish(args, errors)
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
    /// Returns [`SecretArgsError::InvalidArgs`] when any value isn't a valid argument definition
    /// or when the arguments break the rules described for [`SecretArgs::from_args`]. The error
    /// lists a [`SecretArgsError::InvalidArg`] for every invalid value followed by every rule the
    /// valid argument definitions break.
    pub fn from_values(values: Vec<Value>) -> Result<Self, SecretArgsError> {
        let mut errors = Vec::new();
        let mut args = Vec::with_capacity(values.len());
        for (index, value) in values.into_iter().enumerate() {
            match SecretArgKind::deserialize(&value) {
                Ok(arg) => args.push(arg),
                Err(_) => errors.push(SecretArgsError::InvalidArg { position: index + 1, value }),
            }
        }
        Self::validate(&args, &mut errors);
        Self::finish(args, errors)
    }

    /// Resolves the argument definitions into the arguments to pass to the `secret` command.
    ///
    /// Static string arguments are passed as-is. The secret name input argument is followed by
    /// `name`. The vault input argument is followed by the vault name when `vault` is defined and
    /// omitted entirely otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the secret to retrieve.
    /// * `vault` - The name of the vault to retrieve the secret from, if any.
    #[must_use]
    pub fn to_command_args(&self, name: &str, vault: Option<&str>) -> Vec<String> {
        let mut command_args = Vec::<String>::with_capacity(self.0.len() + 2);
        for arg in &self.0 {
            match arg {
                SecretArgKind::String(s) => {
                    command_args.push(s.clone());
                },
                SecretArgKind::Name { name_arg } => {
                    command_args.push(name_arg.clone());
                    command_args.push(name.to_string());
                },
                SecretArgKind::Vault { vault_arg } => {
                    if let Some(value) = vault {
                        command_args.push(vault_arg.clone());
                        command_args.push(value.to_string());
                    }
                },
            }
        }

        command_args
    }

    /// Collects every rule the argument definitions break into `errors`.
    ///
    /// The arguments must define the secret name input argument exactly once and the vault input
    /// argument at most once.
    ///
    /// # Arguments
    ///
    /// * `args` - The argument definitions to validate.
    /// * `errors` - The vector to add the detected rule violations to.
    fn validate(args: &[SecretArgKind], errors: &mut Vec<SecretArgsError>) {
        let name_arg_count = args.iter().filter(|arg| matches!(arg, SecretArgKind::Name { .. })).count();
        let vault_arg_count = args.iter().filter(|arg| matches!(arg, SecretArgKind::Vault { .. })).count();

        match name_arg_count {
            0 => errors.push(SecretArgsError::MissingNameArg),
            1 => {},
            count => errors.push(SecretArgsError::MultipleNameArgs { count }),
        }
        if vault_arg_count > 1 {
            errors.push(SecretArgsError::MultipleVaultArgs { count: vault_arg_count });
        }
    }

    /// Returns the arguments when no errors were collected and the collected errors otherwise.
    ///
    /// # Arguments
    ///
    /// * `args` - The validated argument definitions.
    /// * `errors` - The rule violations collected while validating the arguments.
    fn finish(args: Vec<SecretArgKind>, errors: Vec<SecretArgsError>) -> Result<Self, SecretArgsError> {
        if errors.is_empty() {
            Ok(Self(args))
        } else {
            Err(SecretArgsError::InvalidArgs { errors })
        }
    }
}

impl TryFrom<Vec<Value>> for SecretArgs {
    type Error = SecretArgsError;

    fn try_from(values: Vec<Value>) -> Result<Self, Self::Error> {
        Self::from_values(values)
    }
}

impl TryFrom<Vec<SecretArgKind>> for SecretArgs {
    type Error = SecretArgsError;

    fn try_from(args: Vec<SecretArgKind>) -> Result<Self, Self::Error> {
        Self::from_args(args)
    }
}

// Treat a reference to `SecretArgs` as a reference to the underlying `Vec`
impl AsRef<Vec<SecretArgKind>> for SecretArgs {
    fn as_ref(&self) -> &Vec<SecretArgKind> {
        &self.0
    }
}

// Enable using non-mutating `Vec` methods directly on `SecretArgs`
impl Deref for SecretArgs {
    type Target = Vec<SecretArgKind>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Enable iterating over the underlying `Vec` to use in ergonomic `for` loops
impl IntoIterator for SecretArgs {
    type Item = SecretArgKind;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Enable iterating over references to the argument definitions without consuming the instance
impl<'a> IntoIterator for &'a SecretArgs {
    type Item = &'a SecretArgKind;
    type IntoIter = std::slice::Iter<'a, SecretArgKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

// Enable comparing the newtype with the underlying `Vec` in either direction
impl PartialEq<Vec<SecretArgKind>> for SecretArgs {
    fn eq(&self, other: &Vec<SecretArgKind>) -> bool {
        &self.0 == other
    }
}

impl PartialEq<SecretArgs> for Vec<SecretArgKind> {
    fn eq(&self, other: &SecretArgs) -> bool {
        self == &other.0
    }
}

// Enable infallibly converting the newtype into the underlying `Vec`
impl From<SecretArgs> for Vec<SecretArgKind> {
    fn from(args: SecretArgs) -> Self {
        args.0
    }
}

// Display the arguments as they appear in a manifest, like `["get",{"nameArg":"--name"}]`
impl Display for SecretArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(self).map_err(|_| std::fmt::Error)?;
        f.write_str(&json)
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
    #[schemars(extend(
        "title" = schema_i18n!("executable.title"),
        "description" = schema_i18n!("executable.description"),
        "markdownDescription" = schema_i18n!("executable.markdownDescription"),
    ))]
    pub executable: String,
    /// The arguments to pass to the command to retrieve a secret. The arguments must define the
    /// secret name input argument exactly once and may define the vault input argument at most
    /// once.
    #[schemars(extend(
        "title" = schema_i18n!("args.title"),
        "description" = schema_i18n!("args.description"),
        "markdownDescription" = schema_i18n!("args.markdownDescription"),
        "allOf" = [
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
        ],
    ))]
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
    pub(crate) fn secret(&self, name: &str, vault: Option<&str>) -> Result<Option<Value>, DscError> {
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
            let args = secret.args.to_command_args(name, vault);
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
                // if the secret is a valid JSON object, we wrap it as a SecureObject
                if let Ok(json_value) = serde_json::from_str::<Value>(&secret) && json_value.is_object() {
                    let secure_object = SecureObject {
                        secure_object: json_value.clone(),
                    };
                    return Ok(Some(serde_json::to_value(&secure_object)?));
                }
                let secure_string = SecureString {
                    secure_string: secret.clone(),
                };
                Ok(Some(serde_json::to_value(&secure_string)?))
            }
        } else {
            Err(DscError::UnsupportedCapability(
                self.type_name.to_string(),
                Capability::Secret.to_string()
            ))
        }
    }
}
