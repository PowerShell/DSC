// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::schemas::dsc_repo::DscRepoSchema;

use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};

use crate::schemas::dsc_repo::schema_i18n;
use crate::settings::DscSettingsScope;

/// A resolved setting field value with the scope it was defined in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DscSettingsResolvedField<T> {
    /// The resolved value for the field.
    pub value: T,
    /// The scope the value was defined in.
    pub scope: DscSettingsScope,
}

impl<T> JsonSchema for DscSettingsResolvedField<T> where T: JsonSchema {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Owned(format!("Resolved{}", T::schema_name()))
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "title": schema_i18n!("title"),
            "description": schema_i18n!("description"),
            "markdownDescription": schema_i18n!("markdownDescription"),
            "readOnly": true,
            "type": "object",
            "required": ["value", "scope"],
            "properties": {
                "value": generator.subschema_for::<T>(),
                "scope": generator.subschema_for::<DscSettingsScope>(),
            },
        })
    }
    fn inline_schema() -> bool {
        true
    }
}

impl<T> DscRepoSchema for DscSettingsResolvedField<T> where T: JsonSchema {
    const SCHEMA_FILE_BASE_NAME: &'static str = "field";
    const SCHEMA_FOLDER_PATH: &'static str = "settings/resolved";
    const SCHEMA_I18N_ROOT_KEY: &'static str = "schemas.settings.resolved.field";
    const SCHEMA_SHOULD_BUNDLE: bool = false;
    fn schema_property_metadata() -> schemars::Schema {
        json_schema!({})
    }
    fn schema_i18n(suffix: &str) -> Result<String, crate::schemas::dsc_repo::DscRepoSchemaMissingTranslationError> {
        let i18n_key = format!("{}.{}", Self::SCHEMA_I18N_ROOT_KEY, suffix);
        if let Some(translated) = crate::_rust_i18n_try_translate(&rust_i18n::locale(), &i18n_key) {
            Ok(translated.into())
        } else {
            Err(crate::schemas::dsc_repo::DscRepoSchemaMissingTranslationError { i18n_key })
        }
    }
}

impl<T> DscSettingsResolvedField<T> where T: JsonSchema {
    /// Creates a new resolved field with the given value and scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    /// - `scope`: The scope for the resolved field as a variant of
    ///   [`DscSettingsScope`].
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the given value and
    /// scope.
    pub fn new(value: T, scope: DscSettingsScope) -> Self {
        Self { value, scope }
    }

    /// Creates a new resolved field with the given value and the [`Default`]
    /// scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the [`Default`] scope
    /// and the given value.
    ///
    /// [`Default`]: crate::settings::DscSettingsScope::Default
    pub fn for_code_default(value: T) -> Self {
        Self { value, scope: DscSettingsScope::Default }
    }

    /// Creates a new resolved field with the given value and the
    /// [`Environment`] scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the [`Environment`]
    /// scope and the given value.
    ///
    /// [`Environment`]: crate::settings::DscSettingsScope::Environment
    pub fn for_environment(value: T) -> Self {
        Self { value, scope: DscSettingsScope::Environment }
    }

    /// Creates a new resolved field with the given value and the
    /// [`CommandLine`] scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the [`CommandLine`]
    /// scope and the given value.
    ///
    /// [`CommandLine`]: crate::settings::DscSettingsScope::CommandLine
    pub fn for_command_line(value: T) -> Self {
        Self { value, scope: DscSettingsScope::CommandLine }
    }

    /// Creates a new resolved field with the given value and the [`Machine`]
    /// scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the [`Machine`] scope
    /// and the given value.
    ///
    /// [`Machine`]: crate::settings::DscSettingsScope::Machine
    pub fn for_machine(value: T) -> Self {
        Self { value, scope: DscSettingsScope::Machine }
    }

    /// Creates a new resolved field with the given value and the [`User`]
    /// scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the [`User`] scope
    /// and the given value.
    ///
    /// [`User`]: crate::settings::DscSettingsScope::User
    pub fn for_user(value: T) -> Self {
        Self { value, scope: DscSettingsScope::User }
    }

    /// Creates a new resolved field with the given value and the [`Workspace`]
    /// scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the [`Workspace`]
    /// scope and the given value.
    ///
    /// [`Workspace`]: crate::settings::DscSettingsScope::Workspace
    pub fn for_workspace(value: T) -> Self {
        Self { value, scope: DscSettingsScope::Workspace }
    }

    /// Creates a new resolved field with the given value and the [`Policy`]
    /// scope.
    ///
    /// # Arguments
    ///
    /// - `value`: The value for the resolved field.
    ///
    /// # Returns
    ///
    /// An instance of [`DscSettingsResolvedField`] with the [`Policy`] scope
    /// and the given value.
    ///
    /// [`Policy`]: crate::settings::DscSettingsScope::Policy
    pub fn for_policy(value: T) -> Self {
        Self { value, scope: DscSettingsScope::Policy }
    }

    /// Returns true if the field is enforced by policy and must not be
    /// overridden by environment variables or CLI options.
    ///
    /// # Returns
    ///
    /// `true` if the field is enforced by policy, `false` otherwise.
    #[must_use]
    pub fn is_policy(&self) -> bool {
        self.scope == DscSettingsScope::Policy
    }
}
