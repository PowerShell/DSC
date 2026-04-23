// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use serde::{Deserialize, Serialize};

/// Represents the authentication type for the SecretStore vault.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Authentication {
    /// No password is required to access the vault.
    None,
    /// A password is required to access the vault.
    Password,
}

impl Default for Authentication {
    fn default() -> Self {
        Self::Password
    }
}

impl std::fmt::Display for Authentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Password => write!(f, "Password"),
        }
    }
}

/// Represents the interaction preference for the SecretStore vault.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Interaction {
    /// The vault will prompt the user for input when needed.
    Prompt,
    /// The vault will never prompt the user; operations that require interaction will fail.
    None,
}

impl Default for Interaction {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for Interaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prompt => write!(f, "Prompt"),
            Self::None => write!(f, "None"),
        }
    }
}

/// Represents the SecretStore vault configuration.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VaultConfig {
    /// The authentication type for the vault.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Authentication>,

    /// The password timeout in seconds. After this time, the vault is locked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_timeout: Option<i32>,

    /// The interaction preference for the vault.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction: Option<Interaction>,

    /// Whether the SecretManagement module is installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_management_installed: Option<bool>,

    /// Whether the SecretStore module is installed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_store_installed: Option<bool>,

    /// Whether the SecretStore vault is registered with SecretManagement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_registered: Option<bool>,

    /// Whether the resource exists / should exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _exist: Option<bool>,
}

/// Represents the type of a secret stored in the vault.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum SecretType {
    String,
    SecureString,
    ByteArray,
    PSCredential,
    Hashtable,
}

impl std::fmt::Display for SecretType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "String"),
            Self::SecureString => write!(f, "SecureString"),
            Self::ByteArray => write!(f, "ByteArray"),
            Self::PSCredential => write!(f, "PSCredential"),
            Self::Hashtable => write!(f, "Hashtable"),
        }
    }
}

/// Represents a secret in the SecretStore vault.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    /// The name of the secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The type of the secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_type: Option<SecretType>,

    /// The value to set for the secret. Only used during set operations.
    /// This field is never returned in get/export output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// The vault name to use. Defaults to the SecretStore default vault.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_name: Option<String>,

    /// Metadata associated with the secret (key-value pairs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Whether the secret exists / should exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _exist: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- Authentication enum ---

    #[test]
    fn authentication_default_is_password() {
        assert_eq!(Authentication::default(), Authentication::Password);
    }

    #[test]
    fn authentication_display() {
        assert_eq!(Authentication::None.to_string(), "None");
        assert_eq!(Authentication::Password.to_string(), "Password");
    }

    #[test]
    fn authentication_serde_roundtrip() {
        let json = json!("None");
        let auth: Authentication = serde_json::from_value(json).unwrap();
        assert_eq!(auth, Authentication::None);
        assert_eq!(serde_json::to_value(&auth).unwrap(), json!("None"));

        let json = json!("Password");
        let auth: Authentication = serde_json::from_value(json).unwrap();
        assert_eq!(auth, Authentication::Password);
        assert_eq!(serde_json::to_value(&auth).unwrap(), json!("Password"));
    }

    // --- Interaction enum ---

    #[test]
    fn interaction_default_is_none() {
        assert_eq!(Interaction::default(), Interaction::None);
    }

    #[test]
    fn interaction_display() {
        assert_eq!(Interaction::Prompt.to_string(), "Prompt");
        assert_eq!(Interaction::None.to_string(), "None");
    }

    #[test]
    fn interaction_serde_roundtrip() {
        let json = json!("Prompt");
        let interaction: Interaction = serde_json::from_value(json).unwrap();
        assert_eq!(interaction, Interaction::Prompt);
        assert_eq!(serde_json::to_value(&interaction).unwrap(), json!("Prompt"));

        let json = json!("None");
        let interaction: Interaction = serde_json::from_value(json).unwrap();
        assert_eq!(interaction, Interaction::None);
        assert_eq!(serde_json::to_value(&interaction).unwrap(), json!("None"));
    }

    // --- SecretType enum ---

    #[test]
    fn secret_type_display() {
        assert_eq!(SecretType::String.to_string(), "String");
        assert_eq!(SecretType::SecureString.to_string(), "SecureString");
        assert_eq!(SecretType::ByteArray.to_string(), "ByteArray");
        assert_eq!(SecretType::PSCredential.to_string(), "PSCredential");
        assert_eq!(SecretType::Hashtable.to_string(), "Hashtable");
    }

    #[test]
    fn secret_type_serde_roundtrip() {
        let variants = vec![
            (SecretType::String, "String"),
            (SecretType::SecureString, "SecureString"),
            (SecretType::ByteArray, "ByteArray"),
            (SecretType::PSCredential, "PSCredential"),
            (SecretType::Hashtable, "Hashtable"),
        ];
        for (variant, expected) in variants {
            let serialized = serde_json::to_value(&variant).unwrap();
            assert_eq!(serialized, json!(expected));
            let deserialized: SecretType = serde_json::from_value(serialized).unwrap();
            assert_eq!(deserialized, variant);
        }
    }

    // --- VaultConfig struct ---

    #[test]
    fn vault_config_default_is_all_none() {
        let config = VaultConfig::default();
        assert!(config.authentication.is_none());
        assert!(config.password_timeout.is_none());
        assert!(config.interaction.is_none());
        assert!(config.secret_management_installed.is_none());
        assert!(config.secret_store_installed.is_none());
        assert!(config.vault_registered.is_none());
        assert!(config._exist.is_none());
    }

    #[test]
    fn vault_config_serializes_empty_when_all_none() {
        let config = VaultConfig::default();
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json, json!({}));
    }

    #[test]
    fn vault_config_uses_camel_case_keys() {
        let config = VaultConfig {
            authentication: Some(Authentication::Password),
            password_timeout: Some(900),
            interaction: Some(Interaction::None),
            secret_management_installed: Some(true),
            secret_store_installed: Some(true),
            vault_registered: Some(true),
            _exist: Some(true),
        };
        let json = serde_json::to_value(&config).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("authentication"));
        assert!(obj.contains_key("passwordTimeout"));
        assert!(obj.contains_key("interaction"));
        assert!(obj.contains_key("secretManagementInstalled"));
        assert!(obj.contains_key("secretStoreInstalled"));
        assert!(obj.contains_key("vaultRegistered"));
        assert!(obj.contains_key("_exist"));
    }

    #[test]
    fn vault_config_serde_roundtrip() {
        let config = VaultConfig {
            authentication: Some(Authentication::None),
            password_timeout: Some(300),
            interaction: Some(Interaction::Prompt),
            secret_management_installed: Some(false),
            secret_store_installed: Some(true),
            vault_registered: Some(false),
            _exist: Some(true),
        };
        let json_str = serde_json::to_string(&config).unwrap();
        let deserialized: VaultConfig = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.authentication, Some(Authentication::None));
        assert_eq!(deserialized.password_timeout, Some(300));
        assert_eq!(deserialized.interaction, Some(Interaction::Prompt));
        assert_eq!(deserialized.secret_management_installed, Some(false));
        assert_eq!(deserialized.secret_store_installed, Some(true));
        assert_eq!(deserialized.vault_registered, Some(false));
        assert_eq!(deserialized._exist, Some(true));
    }

    #[test]
    fn vault_config_deserializes_from_partial_json() {
        let json = json!({"authentication": "Password", "passwordTimeout": 600});
        let config: VaultConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.authentication, Some(Authentication::Password));
        assert_eq!(config.password_timeout, Some(600));
        assert!(config.interaction.is_none());
        assert!(config.secret_management_installed.is_none());
        assert!(config._exist.is_none());
    }

    #[test]
    fn vault_config_skips_none_fields_in_serialization() {
        let config = VaultConfig {
            authentication: Some(Authentication::Password),
            password_timeout: None,
            interaction: None,
            secret_management_installed: None,
            secret_store_installed: None,
            vault_registered: None,
            _exist: None,
        };
        let json = serde_json::to_value(&config).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert!(obj.contains_key("authentication"));
    }

    // --- Secret struct ---

    #[test]
    fn secret_default_is_all_none() {
        let secret = Secret::default();
        assert!(secret.name.is_none());
        assert!(secret.secret_type.is_none());
        assert!(secret.value.is_none());
        assert!(secret.vault_name.is_none());
        assert!(secret.metadata.is_none());
        assert!(secret._exist.is_none());
    }

    #[test]
    fn secret_serializes_empty_when_all_none() {
        let secret = Secret::default();
        let json = serde_json::to_value(&secret).unwrap();
        assert_eq!(json, json!({}));
    }

    #[test]
    fn secret_uses_camel_case_keys() {
        let secret = Secret {
            name: Some("test".to_string()),
            secret_type: Some(SecretType::String),
            value: Some("val".to_string()),
            vault_name: Some("SecretStore".to_string()),
            metadata: Some(json!({"key": "value"})),
            _exist: Some(true),
        };
        let json = serde_json::to_value(&secret).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("secretType"));
        assert!(obj.contains_key("value"));
        assert!(obj.contains_key("vaultName"));
        assert!(obj.contains_key("metadata"));
        assert!(obj.contains_key("_exist"));
    }

    #[test]
    fn secret_serde_roundtrip() {
        let secret = Secret {
            name: Some("MySecret".to_string()),
            secret_type: Some(SecretType::SecureString),
            value: Some("s3cret!".to_string()),
            vault_name: Some("SecretStore".to_string()),
            metadata: Some(json!({"env": "prod", "owner": "admin"})),
            _exist: Some(true),
        };
        let json_str = serde_json::to_string(&secret).unwrap();
        let deserialized: Secret = serde_json::from_str(&json_str).unwrap();
        assert_eq!(deserialized.name, Some("MySecret".to_string()));
        assert_eq!(deserialized.secret_type, Some(SecretType::SecureString));
        assert_eq!(deserialized.value, Some("s3cret!".to_string()));
        assert_eq!(deserialized.vault_name, Some("SecretStore".to_string()));
        assert_eq!(deserialized.metadata, Some(json!({"env": "prod", "owner": "admin"})));
        assert_eq!(deserialized._exist, Some(true));
    }

    #[test]
    fn secret_deserializes_from_minimal_json() {
        let json = json!({"name": "OnlyName"});
        let secret: Secret = serde_json::from_value(json).unwrap();
        assert_eq!(secret.name, Some("OnlyName".to_string()));
        assert!(secret.secret_type.is_none());
        assert!(secret.value.is_none());
        assert!(secret.vault_name.is_none());
        assert!(secret.metadata.is_none());
        assert!(secret._exist.is_none());
    }

    #[test]
    fn secret_skips_none_fields_in_serialization() {
        let secret = Secret {
            name: Some("test".to_string()),
            _exist: Some(false),
            ..Default::default()
        };
        let json = serde_json::to_value(&secret).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("_exist"));
    }

    #[test]
    fn secret_metadata_with_nested_values() {
        let secret = Secret {
            name: Some("test".to_string()),
            metadata: Some(json!({"tag": "test", "count": "42"})),
            ..Default::default()
        };
        let json = serde_json::to_value(&secret).unwrap();
        let meta = json.get("metadata").unwrap();
        assert_eq!(meta.get("tag").unwrap(), "test");
        assert_eq!(meta.get("count").unwrap(), "42");
    }
}
