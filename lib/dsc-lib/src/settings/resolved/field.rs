use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::settings::DscSettingsScope;

/// A resolved setting field value with the scope it was defined in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DscSettingsResolvedField<T> {
    /// The resolved value for the field.
    pub value: T,
    /// The scope the value was defined in.
    pub scope: DscSettingsScope,
}

impl<T> DscSettingsResolvedField<T> {
    /// Creates a new resolved field with the given value and scope.
    pub fn new(value: T, scope: DscSettingsScope) -> Self {
        Self { value, scope }
    }
    /// Returns true if the field is enforced by policy and must not be overridden
    /// by environment variables or CLI options.
    #[must_use]
    pub fn is_policy(&self) -> bool {
        self.scope == DscSettingsScope::Policy
    }
}
