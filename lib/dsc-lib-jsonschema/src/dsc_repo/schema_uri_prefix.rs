// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Defines the URI prefix for the hosted schemas.
///
/// While the schemas are currently hosted in the GitHub repository, DSC provides the shortened
/// `aka.ms` link for convenience. Using this enum simplifies migrating to a new URI for schemas
/// in the future.
#[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum SchemaUriPrefix {
    /// Defines the shortened link URI prefix as `https://aka.ms/dsc/schemas`.
    #[default]
    AkaDotMs,
    /// Defines the canonical URI prefix as `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas`.
    Github,
}

impl std::fmt::Display for SchemaUriPrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AkaDotMs => write!(f, "https://aka.ms/dsc/schemas"),
            Self::Github => write!(f, "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas"),
        }
    }
}

impl SchemaUriPrefix {
    /// Returns every known URI prefix for convenient iteration.
    #[must_use]
    pub fn all() -> Vec<SchemaUriPrefix> {
        vec![
            Self::AkaDotMs,
            Self::Github,
        ]
    }
}
