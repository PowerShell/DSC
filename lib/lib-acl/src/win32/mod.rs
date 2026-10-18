// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Win32-backed operations for reading and writing access control lists. Only compiled on
//! Windows; see [`crate::model`] for the portable typed model these operations read and write.

pub mod acl;
pub mod privilege;
pub mod security_descriptor;
pub mod sid;

pub use privilege::{SE_SECURITY_NAME, enable_security_privilege};
pub use security_descriptor::{
    build_self_relative_security_descriptor, normalize, read_access_control_list,
    read_security_information,
};
pub use sid::{resolve_principal, sid_to_string};
