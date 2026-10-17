// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use lib_acl::{AccessControlList, AccessMask};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum RegistryAccessRight {
    /// `KEY_QUERY_VALUE`: Required to query the values of a registry key.
    QueryValue,
    /// `KEY_SET_VALUE`: Required to create, delete, or set a registry value.
    SetValue,
    /// `KEY_CREATE_SUB_KEY`: Required to create a subkey of a registry key.
    CreateSubKey,
    /// `KEY_ENUMERATE_SUB_KEYS`: Required to enumerate the subkeys of a registry key.
    EnumerateSubKeys,
    /// `KEY_NOTIFY`: Required to request change notifications for a registry key or for
    /// subkeys of a registry key.
    Notify,
    /// `KEY_CREATE_LINK`: Reserved for system use.
    CreateLink,
    /// `KEY_WOW64_64KEY`: Indicates that an application on 64-bit Windows should operate on
    /// the 64-bit registry view.
    Wow6464Key,
    /// `KEY_WOW64_32KEY`: Indicates that an application on 64-bit Windows should operate on
    /// the 32-bit registry view.
    Wow6432Key,
    /// `DELETE`: Required to delete the registry key.
    Delete,
    /// `READ_CONTROL`: Required to read the information in the registry key's security
    /// descriptor, not including the information in the system access control list (SACL).
    ReadPermissions,
    /// `WRITE_DAC`: Required to modify the discretionary access control list (DACL) in the
    /// registry key's security descriptor.
    ChangePermissions,
    /// `WRITE_OWNER`: Required to change the owner in the registry key's security descriptor.
    TakeOwnership,
    /// `SYNCHRONIZE`: The right to use the registry key for synchronization.
    Synchronize,
    /// `KEY_READ`: A composite of [`QueryValue`](Self::QueryValue),
    /// [`EnumerateSubKeys`](Self::EnumerateSubKeys), [`Notify`](Self::Notify), and
    /// [`ReadPermissions`](Self::ReadPermissions).
    Read,
    /// `KEY_WRITE`: A composite of [`SetValue`](Self::SetValue),
    /// [`CreateSubKey`](Self::CreateSubKey), and [`ReadPermissions`](Self::ReadPermissions).
    Write,
    /// `KEY_EXECUTE`: Equivalent to [`Read`](Self::Read).
    Execute,
    /// `KEY_ALL_ACCESS`: A composite of every standard and registry-specific right.
    FullControl,
}

impl AccessMask for RegistryAccessRight {
    fn bits(self) -> u32 {
        match self {
            Self::QueryValue => 0x0000_0001,
            Self::SetValue => 0x0000_0002,
            Self::CreateSubKey => 0x0000_0004,
            Self::EnumerateSubKeys => 0x0000_0008,
            Self::Notify => 0x0000_0010,
            Self::CreateLink => 0x0000_0020,
            Self::Wow6464Key => 0x0000_0100,
            Self::Wow6432Key => 0x0000_0200,
            Self::Delete => 0x0001_0000,
            Self::ReadPermissions => 0x0002_0000,
            Self::ChangePermissions => 0x0004_0000,
            Self::TakeOwnership => 0x0008_0000,
            Self::Synchronize => 0x0010_0000,
            Self::Read | Self::Execute => 0x0002_0019,
            Self::Write => 0x0002_0006,
            Self::FullControl => 0x000F_003F,
        }
    }

    fn all() -> &'static [Self] {
        &[
            Self::QueryValue,
            Self::SetValue,
            Self::CreateSubKey,
            Self::EnumerateSubKeys,
            Self::Notify,
            Self::CreateLink,
            Self::Wow6464Key,
            Self::Wow6432Key,
            Self::Delete,
            Self::ReadPermissions,
            Self::ChangePermissions,
            Self::TakeOwnership,
            Self::Synchronize,
            Self::Read,
            Self::Write,
            Self::Execute,
            Self::FullControl,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RegistryValueData {
    String(String),
    ExpandString(String),
    Binary(Vec<u8>),
    DWord(u32),
    MultiString(Vec<String>),
    QWord(u64),
    None,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename = "Registry", deny_unknown_fields)]
pub struct Registry {
    /// The path to the registry key.
    #[serde(rename = "keyPath")]
    pub key_path: String,
    /// The information from a config set --what-if operation.
    #[serde(rename = "_metadata", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    /// The name of the registry value.
    #[serde(rename = "valueName", skip_serializing_if = "Option::is_none")]
    pub value_name: Option<String>,
    /// The data of the registry value.
    #[serde(rename = "valueData", skip_serializing_if = "Option::is_none")]
    pub value_data: Option<RegistryValueData>,
    /// The registry key's access control list (discretionary and/or system).
    #[serde(rename = "accessControlList", skip_serializing_if = "Option::is_none")]
    pub access_control_list: Option<AccessControlList<RegistryAccessRight>>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
    /// Optional path to an offline registry hive file. When specified, operations
    /// are performed against the offline hive instead of the live system registry.
    #[serde(rename = "registryFilePath", skip_serializing_if = "Option::is_none")]
    pub registry_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub what_if: Option<Vec<String>>,
}
