// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use lib_acl::{AccessControlList, AccessMask};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileContent {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha512: Option<String>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,
}

/// The access rights understood by the Windows filesystem (files and folders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FileAccessRight {
    /// `FILE_READ_DATA` (`FILE_LIST_DIRECTORY` for a folder): read the file's data, or list a
    /// folder's contents.
    ReadData,
    /// `FILE_WRITE_DATA` (`FILE_ADD_FILE` for a folder): write the file's data, or create a
    /// file in a folder.
    WriteData,
    /// `FILE_APPEND_DATA` (`FILE_ADD_SUBDIRECTORY` for a folder): append data to the file, or
    /// create a subdirectory in a folder.
    AppendData,
    /// `FILE_READ_EA`: read the file or folder's extended attributes.
    ReadExtendedAttributes,
    /// `FILE_WRITE_EA`: write the file or folder's extended attributes.
    WriteExtendedAttributes,
    /// `FILE_EXECUTE` (`FILE_TRAVERSE` for a folder): run the file, or traverse through a
    /// folder without the `FILE_LIST_DIRECTORY` right.
    ExecuteFile,
    /// `FILE_DELETE_CHILD`: delete a folder's children, regardless of the permissions on the
    /// child itself.
    DeleteChild,
    /// `FILE_READ_ATTRIBUTES`: read the file or folder's basic attributes.
    ReadAttributes,
    /// `FILE_WRITE_ATTRIBUTES`: write the file or folder's basic attributes.
    WriteAttributes,
    /// `DELETE`: delete the file or folder.
    Delete,
    /// `READ_CONTROL`: read the information in the file or folder's security descriptor, not
    /// including the information in the system access control list (SACL).
    ReadPermissions,
    /// `WRITE_DAC`: modify the discretionary access control list (DACL) in the file or
    /// folder's security descriptor.
    ChangePermissions,
    /// `WRITE_OWNER`: change the owner in the file or folder's security descriptor.
    TakeOwnership,
    /// `SYNCHRONIZE`: the right to use the file or folder handle for synchronization.
    Synchronize,
    /// `FILE_GENERIC_READ`: a composite of [`ReadData`](Self::ReadData),
    /// [`ReadAttributes`](Self::ReadAttributes),
    /// [`ReadExtendedAttributes`](Self::ReadExtendedAttributes),
    /// [`ReadPermissions`](Self::ReadPermissions), and [`Synchronize`](Self::Synchronize).
    Read,
    /// `FILE_GENERIC_WRITE`: a composite of [`WriteData`](Self::WriteData),
    /// [`AppendData`](Self::AppendData), [`WriteAttributes`](Self::WriteAttributes),
    /// [`WriteExtendedAttributes`](Self::WriteExtendedAttributes),
    /// [`ReadPermissions`](Self::ReadPermissions), and [`Synchronize`](Self::Synchronize).
    Write,
    /// The standard "Read & execute" permission: a composite of [`Read`](Self::Read) and
    /// [`ExecuteFile`](Self::ExecuteFile).
    ReadAndExecute,
    /// The standard "Modify" permission: a composite of
    /// [`ReadAndExecute`](Self::ReadAndExecute), [`Write`](Self::Write), and
    /// [`Delete`](Self::Delete).
    Modify,
    /// `FILE_ALL_ACCESS`: a composite of every standard and file-specific right.
    FullControl,
    /// `GENERIC_READ`: the generic read right. Windows maps this to [`Read`](Self::Read) for
    /// file and folder objects, but the raw bit can still appear in an existing access control
    /// entry's mask, so it must be representable rather than silently discarded when read back.
    GenericRead,
    /// `GENERIC_WRITE`: the generic write right, mapped by Windows to
    /// [`Write`](Self::Write) for file and folder objects.
    GenericWrite,
    /// `GENERIC_EXECUTE`: the generic execute right, mapped by Windows to
    /// [`ExecuteFile`](Self::ExecuteFile)-equivalent access for file and folder objects.
    GenericExecute,
    /// `GENERIC_ALL`: the generic all-access right, mapped by Windows to
    /// [`FullControl`](Self::FullControl) for file and folder objects.
    GenericAll,
}

impl AccessMask for FileAccessRight {
    fn bits(self) -> u32 {
        match self {
            Self::ReadData => 0x0000_0001,
            Self::WriteData => 0x0000_0002,
            Self::AppendData => 0x0000_0004,
            Self::ReadExtendedAttributes => 0x0000_0008,
            Self::WriteExtendedAttributes => 0x0000_0010,
            Self::ExecuteFile => 0x0000_0020,
            Self::DeleteChild => 0x0000_0040,
            Self::ReadAttributes => 0x0000_0080,
            Self::WriteAttributes => 0x0000_0100,
            Self::Delete => 0x0001_0000,
            Self::ReadPermissions => 0x0002_0000,
            Self::ChangePermissions => 0x0004_0000,
            Self::TakeOwnership => 0x0008_0000,
            Self::Synchronize => 0x0010_0000,
            Self::Read => 0x0012_0089,
            Self::Write => 0x0012_0116,
            Self::ReadAndExecute => 0x0012_00A9,
            Self::Modify => 0x0013_01BF,
            Self::FullControl => 0x001F_01FF,
            Self::GenericExecute => 0x2000_0000,
            Self::GenericWrite => 0x4000_0000,
            Self::GenericRead => 0x8000_0000,
            Self::GenericAll => 0x1000_0000,
        }
    }

    fn all() -> &'static [Self] {
        &[
            Self::ReadData,
            Self::WriteData,
            Self::AppendData,
            Self::ReadExtendedAttributes,
            Self::WriteExtendedAttributes,
            Self::ExecuteFile,
            Self::DeleteChild,
            Self::ReadAttributes,
            Self::WriteAttributes,
            Self::Delete,
            Self::ReadPermissions,
            Self::ChangePermissions,
            Self::TakeOwnership,
            Self::Synchronize,
            Self::Read,
            Self::Write,
            Self::ReadAndExecute,
            Self::Modify,
            Self::FullControl,
            Self::GenericRead,
            Self::GenericWrite,
            Self::GenericExecute,
            Self::GenericAll,
        ]
    }

    fn map_generic_bits(bits: u32) -> u32 {
        let mut mapped = bits;
        for (generic, specific) in [
            (0x1000_0000, Self::FullControl.bits()),
            (0x2000_0000, 0x0012_00A0),
            (0x4000_0000, Self::Write.bits()),
            (0x8000_0000, Self::Read.bits()),
        ] {
            if mapped & generic != 0 {
                mapped = (mapped & !generic) | specific;
            }
        }
        mapped
    }
}

/// The discretionary and/or system access control list of a local file or folder.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileAccessControlList {
    pub path: String,
    /// The discretionary and/or system access control list. At least one section must be present
    /// unless `_purge` is explicitly true, which clears omitted sections.
    #[serde(rename = "accessControlList", skip_serializing_if = "Option::is_none")]
    pub access_control_list: Option<AccessControlList<FileAccessRight>>,
    #[serde(rename = "_inDesiredState", skip_serializing_if = "Option::is_none")]
    pub in_desired_state: Option<bool>,
}
