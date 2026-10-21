// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::AclError;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// A validated Windows security principal.
///
/// Principals deserialize from either a SID or an account/group name. On Windows, construction
/// resolves the input immediately and stores its canonical SID plus an optional display name.
/// Serialization and display always emit the SID.
#[derive(Debug, Clone)]
pub struct Principal {
    sid: String,
    display_name: Option<String>,
}

impl Principal {
    /// Creates a principal from a SID string.
    ///
    /// # Errors
    ///
    /// Returns [`AclError::InvalidSid`] when `text` is not a valid SID.
    #[cfg(windows)]
    pub fn from_sid(text: &str) -> Result<Self, AclError> {
        Self::reject_nul(text)?;
        let sid = crate::win32::sid::parse_sid(text)?;
        Self::from_owned_sid(&sid)
    }

    /// Returns an error because SID validation requires Windows.
    #[cfg(not(windows))]
    pub fn from_sid(text: &str) -> Result<Self, AclError> {
        Err(Self::windows_only_error(text))
    }

    /// Creates a principal from an account or group name.
    ///
    /// # Errors
    ///
    /// Returns [`AclError::UnresolvablePrincipal`] when `text` cannot be resolved.
    #[cfg(windows)]
    pub fn from_name(text: &str) -> Result<Self, AclError> {
        Self::reject_nul(text)?;
        let sid = crate::win32::sid::lookup_account_name(text)?;
        Self::from_owned_sid(&sid)
    }

    /// Returns an error because account and group lookup requires Windows.
    #[cfg(not(windows))]
    pub fn from_name(text: &str) -> Result<Self, AclError> {
        Err(Self::windows_only_error(text))
    }

    /// Returns the resolved account or group name when Windows can provide one.
    #[must_use]
    pub fn display_name(&self) -> Option<String> {
        self.display_name.clone()
    }

    /// Returns the canonical SID string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.sid
    }

    #[cfg(windows)]
    fn reject_nul(text: &str) -> Result<(), AclError> {
        if text.contains('\0') {
            Err(AclError::UnresolvablePrincipal {
                principal: text.to_string(),
                reason: rust_i18n::t!("principal.nulCharacter").to_string(),
            })
        } else {
            Ok(())
        }
    }

    #[cfg(not(windows))]
    fn windows_only_error(text: &str) -> AclError {
        AclError::UnresolvablePrincipal {
            principal: text.to_string(),
            reason: rust_i18n::t!("principal.windowsOnly").to_string(),
        }
    }

    #[cfg(windows)]
    fn from_owned_sid(sid: &crate::win32::sid::OwnedSid) -> Result<Self, AclError> {
        // SAFETY: `sid` owns a valid SID buffer for the duration of both calls.
        let sid_text = unsafe { crate::win32::sid::sid_to_string(sid.as_psid()) }?;
        // SAFETY: same as above. Account lookup failure only means no display name is available.
        let display_name = unsafe { crate::win32::sid::sid_to_account_name(sid.as_psid()) };
        Ok(Self {
            sid: sid_text,
            display_name,
        })
    }

    /// Creates a principal from a SID pointer supplied by a Windows security descriptor.
    ///
    /// # Safety
    ///
    /// `sid` must point to a valid SID for the duration of this call.
    #[cfg(windows)]
    pub(crate) unsafe fn from_psid(sid: windows::Win32::Security::PSID) -> Result<Self, AclError> {
        // SAFETY: upheld by the caller.
        let sid_text = unsafe { crate::win32::sid::sid_to_string(sid) }?;
        // SAFETY: upheld by the caller. A failed account lookup is represented by `None`.
        let display_name = unsafe { crate::win32::sid::sid_to_account_name(sid) };
        Ok(Self {
            sid: sid_text,
            display_name,
        })
    }
}

#[cfg(windows)]
impl TryFrom<&str> for Principal {
    type Error = AclError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if crate::win32::sid::looks_like_sid(value) {
            Self::from_sid(value)
        } else {
            Self::from_name(value)
        }
    }
}

#[cfg(not(windows))]
impl TryFrom<&str> for Principal {
    type Error = AclError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Err(Self::windows_only_error(value))
    }
}

impl TryFrom<String> for Principal {
    type Error = AclError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Display for Principal {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.sid)
    }
}

impl PartialEq for Principal {
    fn eq(&self, other: &Self) -> bool {
        self.sid == other.sid
    }
}

impl Eq for Principal {}

impl Hash for Principal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sid.hash(state);
    }
}

impl PartialEq<str> for Principal {
    fn eq(&self, other: &str) -> bool {
        self.sid.eq_ignore_ascii_case(other)
            || self
                .display_name
                .as_deref()
                .is_some_and(|name| name.eq_ignore_ascii_case(other))
    }
}

impl PartialEq<&str> for Principal {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<String> for Principal {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Principal> for str {
    fn eq(&self, other: &Principal) -> bool {
        other == self
    }
}

impl PartialEq<Principal> for &str {
    fn eq(&self, other: &Principal) -> bool {
        other == *self
    }
}

impl PartialEq<Principal> for String {
    fn eq(&self, other: &Principal) -> bool {
        other == self
    }
}

impl Serialize for Principal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.sid)
    }
}

impl<'de> Deserialize<'de> for Principal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

impl JsonSchema for Principal {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Principal".into()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        String::json_schema(generator)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn sid_construction_is_canonical_and_serializes_as_a_string() {
        let principal = Principal::from_sid("s-1-5-18").unwrap();

        assert_eq!(principal.as_str(), "S-1-5-18");
        assert_eq!(serde_json::to_string(&principal).unwrap(), "\"S-1-5-18\"");
    }

    #[test]
    fn name_construction_caches_a_display_name_and_compares_by_name_or_sid() {
        let principal = Principal::from_name("SYSTEM").unwrap();
        let display_name = principal.display_name().unwrap();

        assert_eq!(principal, "S-1-5-18");
        assert_eq!(display_name, principal);
    }

    #[test]
    fn deserialization_rejects_embedded_nul() {
        let error = serde_json::from_str::<Principal>("\"Everyone\\u0000DSC-Invalid\"").unwrap_err();

        assert!(error.to_string().contains("must not contain NUL"));
    }
}
