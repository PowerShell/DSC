// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Reads the owner and discretionary access control list of a Windows path into the security
//! model.

use core::ffi::c_void;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr;

use rust_i18n::t;
use windows::Win32::Foundation::{
    CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER, ERROR_PATH_NOT_FOUND, HANDLE,
    HLOCAL, LocalFree, WIN32_ERROR,
};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, GetNamedSecurityInfoW, SE_FILE_OBJECT,
};
use windows::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACL, DACL_SECURITY_INFORMATION, EqualSid, GetAce, GetLengthSid,
    GetTokenInformation, IsWellKnownSid, OWNER_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
    TOKEN_QUERY, TOKEN_USER, TokenUser, WinBuiltinAdministratorsSid, WinLocalSystemSid,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{PCWSTR, PWSTR};

use crate::model::{self, Principal, PrincipalKind, SecurityModel, ace};
use crate::{PathSecurityError, PathSecurityScope, path_string};

/// Owns a security descriptor allocated by the system and frees it on drop.
struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        // SAFETY: The descriptor was allocated with LocalAlloc by the system and is freed exactly
        // once here.
        unsafe {
            let _ = LocalFree(Some(HLOCAL(self.0.0)));
        }
    }
}

/// Owns a string allocated by the system and frees it on drop.
struct LocalString(PWSTR);

impl Drop for LocalString {
    fn drop(&mut self) {
        // SAFETY: ConvertSidToStringSidW allocates the string with LocalAlloc.
        unsafe {
            let _ = LocalFree(Some(HLOCAL(self.0.0.cast::<c_void>())));
        }
    }
}

/// Owns an access token handle and closes it on drop.
struct TokenHandle(HANDLE);

impl Drop for TokenHandle {
    fn drop(&mut self) {
        // SAFETY: The handle was returned by OpenProcessToken and is closed exactly once here.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// A SID copied into owned, correctly aligned memory so it outlives the buffer it came from.
struct OwnedSid(Vec<u32>);

impl OwnedSid {
    /// Returns a pointer to the SID for use with the Win32 SID functions.
    ///
    /// # Returns
    ///
    /// A pointer that stays valid for as long as this value lives.
    fn as_psid(&self) -> PSID {
        PSID(self.0.as_ptr().cast_mut().cast::<c_void>())
    }
}

/// Converts a path to a null terminated UTF 16 string for the Win32 APIs.
///
/// # Arguments
///
/// 1. `path`: the path to convert.
///
/// # Returns
///
/// The UTF 16 code units of the path followed by a terminating zero.
fn to_wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(once(0)).collect()
}

/// Formats a failed Win32 call for the reason of a [`PathSecurityError::SecurityInfoRead`].
///
/// # Arguments
///
/// 1. `operation`: the name of the Win32 function that failed.
/// 1. `code`: the Win32 error code.
///
/// # Returns
///
/// The localized description of the failure.
fn win32_failure(operation: &str, code: u32) -> String {
    t!("error.win32", operation = operation, code = code).to_string()
}

/// Extracts the Win32 error code from a `windows` crate error.
///
/// # Arguments
///
/// 1. `error`: the error returned by the `windows` crate.
///
/// # Returns
///
/// The Win32 error code when the error wraps one, otherwise the raw HRESULT value.
fn error_code(error: &windows::core::Error) -> u32 {
    WIN32_ERROR::from_error(error).map_or_else(|| error.code().0 as u32, |win32| win32.0)
}

/// Builds a read failure for a path from an operation name and a `windows` crate error.
///
/// # Arguments
///
/// 1. `path`: the path the operation was reading.
/// 1. `operation`: the name of the Win32 function that failed.
/// 1. `error`: the error returned by the `windows` crate.
///
/// # Returns
///
/// A [`PathSecurityError::SecurityInfoRead`] describing the failure.
fn read_failure(path: &Path, operation: &str, error: &windows::core::Error) -> PathSecurityError {
    PathSecurityError::SecurityInfoRead {
        path: path_string(path),
        reason: win32_failure(operation, error_code(error)),
    }
}

/// Converts a SID to its string form.
///
/// # Arguments
///
/// 1. `psid`: a pointer to a valid SID.
/// 1. `path`: the path being validated, used in the error when the conversion fails.
///
/// # Errors
///
/// Returns [`PathSecurityError::SecurityInfoRead`] when the system cannot convert the SID.
fn sid_to_string(psid: PSID, path: &Path) -> Result<String, PathSecurityError> {
    let mut raw = PWSTR::null();
    // SAFETY: psid points to a valid SID and raw receives a LocalAlloc string owned by
    // LocalString.
    unsafe { ConvertSidToStringSidW(psid, &mut raw) }
        .map_err(|error| read_failure(path, "ConvertSidToStringSidW", &error))?;
    let text = LocalString(raw);
    // SAFETY: The system returned a null terminated UTF 16 string.
    unsafe { text.0.to_string() }.map_err(|error| PathSecurityError::SecurityInfoRead {
        path: path_string(path),
        reason: error.to_string(),
    })
}

/// Classifies a SID by how the scopes trust it.
///
/// # Arguments
///
/// 1. `psid`: a pointer to a valid SID.
/// 1. `current_user`: the SID of the current user when the user scope is being evaluated.
/// 1. `path`: the path being validated, used in the error when the SID cannot be converted.
///
/// # Errors
///
/// Returns [`PathSecurityError::SecurityInfoRead`] when the SID cannot be converted to a string.
fn classify_sid(psid: PSID, current_user: Option<PSID>, path: &Path) -> Result<Principal, PathSecurityError> {
    let id = sid_to_string(psid, path)?;
    // SAFETY: psid and current_user point to valid SIDs for the duration of the calls.
    let kind = if unsafe { IsWellKnownSid(psid, WinLocalSystemSid) }.as_bool() {
        PrincipalKind::LocalSystem
    } else if unsafe { IsWellKnownSid(psid, WinBuiltinAdministratorsSid) }.as_bool() {
        PrincipalKind::Administrators
    } else if matches!(current_user, Some(current) if unsafe { EqualSid(psid, current) }.is_ok()) {
        PrincipalKind::CurrentUser
    } else {
        PrincipalKind::Other
    };
    Ok(Principal::new(kind, id))
}

/// Reads the SID of the account the current process runs as.
///
/// # Arguments
///
/// 1. `path`: the path being validated, used in the error when the token cannot be read.
///
/// # Errors
///
/// Returns [`PathSecurityError::SecurityInfoRead`] when the process token cannot be opened or
/// queried.
fn current_user_sid(path: &Path) -> Result<OwnedSid, PathSecurityError> {
    let mut raw_token = HANDLE(ptr::null_mut());
    // SAFETY: GetCurrentProcess returns a pseudo handle that needs no cleanup, and raw_token
    // receives a real handle owned by TokenHandle.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw_token) }
        .map_err(|error| read_failure(path, "OpenProcessToken", &error))?;
    let token = TokenHandle(raw_token);

    let mut needed = 0_u32;
    // SAFETY: The first call passes no buffer so the system reports the required size.
    match unsafe { GetTokenInformation(token.0, TokenUser, None, 0, &mut needed) } {
        Ok(()) => {
            return Err(PathSecurityError::SecurityInfoRead {
                path: path_string(path),
                reason: win32_failure("GetTokenInformation", 0),
            });
        }
        Err(error) if WIN32_ERROR::from_error(&error) == Some(ERROR_INSUFFICIENT_BUFFER) => {}
        Err(error) => return Err(read_failure(path, "GetTokenInformation", &error)),
    }

    let mut buffer = vec![0_u64; (needed as usize).div_ceil(size_of::<u64>())];
    // SAFETY: The buffer has the size the system requested and is aligned for TOKEN_USER.
    unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buffer.as_mut_ptr().cast::<c_void>()),
            needed,
            &mut needed,
        )
    }
    .map_err(|error| read_failure(path, "GetTokenInformation", &error))?;

    // SAFETY: The system filled the buffer with a TOKEN_USER whose SID points inside the buffer.
    let token_user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
    let psid = token_user.User.Sid;
    // SAFETY: The SID is valid for as long as the buffer lives.
    let length = unsafe { GetLengthSid(psid) } as usize;
    let mut words = vec![0_u32; length.div_ceil(size_of::<u32>())];
    // SAFETY: Both regions are valid for length bytes and do not overlap.
    unsafe {
        ptr::copy_nonoverlapping(psid.0.cast::<u8>().cast_const(), words.as_mut_ptr().cast::<u8>(), length);
    }
    Ok(OwnedSid(words))
}

/// Reads the owner and access control entries of one path into a security model.
///
/// A symbolic link or junction at the end of the path is not followed, so for such a path the
/// model describes the link itself and not the object the link points to.
///
/// # Arguments
///
/// 1. `path`: the absolute path of the file, folder, or link.
/// 1. `scope`: the scope whose trust rules apply. The current user SID is only read for the
///    user scope.
///
/// # Errors
///
/// Returns [`PathSecurityError::PathNotFound`] when the path does not exist,
/// [`PathSecurityError::NullDacl`] when the path has no discretionary access control list, and
/// [`PathSecurityError::SecurityInfoRead`] when any Win32 call fails or the owner is missing.
fn read_security_model(path: &Path, scope: PathSecurityScope) -> Result<SecurityModel, PathSecurityError> {
    let wide = to_wide(path);
    let mut owner = PSID(ptr::null_mut());
    let mut dacl: *mut ACL = ptr::null_mut();
    let mut raw_descriptor = PSECURITY_DESCRIPTOR(ptr::null_mut());
    // SAFETY: wide is a null terminated string and the out pointers are writable locals.
    let status = unsafe {
        GetNamedSecurityInfoW(
            PCWSTR(wide.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
            Some(&mut owner),
            None,
            Some(&mut dacl),
            None,
            &mut raw_descriptor,
        )
    };
    if status == ERROR_FILE_NOT_FOUND || status == ERROR_PATH_NOT_FOUND {
        return Err(PathSecurityError::PathNotFound {
            path: path_string(path),
        });
    }
    if status.is_err() {
        return Err(PathSecurityError::SecurityInfoRead {
            path: path_string(path),
            reason: win32_failure("GetNamedSecurityInfoW", status.0),
        });
    }
    // The owner and DACL pointers reference memory inside the descriptor, so the guard must stay
    // alive until the model is built.
    let _descriptor = SecurityDescriptor(raw_descriptor);

    if dacl.is_null() {
        return Err(PathSecurityError::NullDacl {
            path: path_string(path),
        });
    }
    if owner.0.is_null() {
        return Err(PathSecurityError::SecurityInfoRead {
            path: path_string(path),
            reason: t!("error.missingOwner").to_string(),
        });
    }

    let current_user = if scope == PathSecurityScope::User {
        Some(current_user_sid(path)?)
    } else {
        None
    };
    let current_psid = current_user.as_ref().map(OwnedSid::as_psid);

    let owner = classify_sid(owner, current_psid, path)?;

    // SAFETY: dacl points to a valid ACL inside the descriptor.
    let ace_count = unsafe { (*dacl).AceCount };
    let mut entries = Vec::with_capacity(usize::from(ace_count));
    for index in 0..u32::from(ace_count) {
        let mut ace_pointer: *mut c_void = ptr::null_mut();
        // SAFETY: index is below the entry count of a valid ACL.
        unsafe { GetAce(dacl, index, &mut ace_pointer) }.map_err(|error| read_failure(path, "GetAce", &error))?;
        let typed_pointer = ace_pointer.cast::<ACCESS_ALLOWED_ACE>().cast_const();
        // SAFETY: Every entry begins with a header followed by the access mask, which is the
        // layout of ACCESS_ALLOWED_ACE.
        let entry = unsafe { ptr::read_unaligned(typed_pointer) };
        let principal = if entry.Header.AceType == ace::ACCESS_ALLOWED_ACE_TYPE {
            // SAFETY: SidStart is the first word of the SID that follows the mask in an entry of
            // this type.
            let sid_pointer = unsafe { &raw const (*typed_pointer).SidStart };
            classify_sid(PSID(sid_pointer.cast_mut().cast::<c_void>()), current_psid, path)?
        } else {
            Principal::new(PrincipalKind::Other, t!("principal.aceType", ace_type = entry.Header.AceType))
        };
        entries.push(ace::classify(entry.Header.AceType, entry.Header.AceFlags, entry.Mask, principal));
    }

    Ok(SecurityModel {
        path: path.to_path_buf(),
        owner,
        entries,
    })
}

/// Validates one existing file, folder, or link without looking at its containing folder.
///
/// A symbolic link or junction at the end of the path is not followed. For such a path the link
/// itself is validated, which matters because a principal that owns a link or has write access
/// to it can point the link somewhere else. The caller resolves links before validating the
/// object they point to.
///
/// # Arguments
///
/// 1. `path`: the absolute path of the file, folder, or link.
/// 1. `scope`: the scope whose trust rules apply.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when the security information cannot be read or when the
/// owner or an access control entry violates the trust rules of the scope.
pub(crate) fn validate_object(path: &Path, scope: PathSecurityScope) -> Result<(), PathSecurityError> {
    let model = read_security_model(path, scope)?;
    model::evaluate(&model, scope)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::windows::fs::{symlink_dir, symlink_file};
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    use dsc_lib_security_context::{SecurityContext, get_security_context};
    use tempfile::TempDir;
    use test_case::test_case;
    use windows::Win32::Security::Authorization::{
        ConvertStringSecurityDescriptorToSecurityDescriptorW, SetNamedSecurityInfoW,
    };
    use windows::Win32::Security::{
        GetSecurityDescriptorDacl, GetSecurityDescriptorOwner, OBJECT_SECURITY_INFORMATION,
        PROTECTED_DACL_SECURITY_INFORMATION,
    };
    use windows::core::{BOOL, HSTRING};

    use super::*;
    use crate::validate_path;

    const SDDL_REVISION: u32 = 1;
    /// Allow entries every test folder carries so SYSTEM and Administrators keep full control.
    const TRUSTED: &str = "(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)";
    /// Inheritable allow entry that makes a folder and its children writable by everyone.
    const EVERYONE_FULL: &str = "(A;OICI;FA;;;WD)";
    const EVERYONE: &str = "S-1-1-0";
    const USERS: &str = "S-1-5-32-545";
    /// The Win32 error for creating a symbolic link without the privilege to do so.
    const ERROR_PRIVILEGE_NOT_HELD: i32 = 1314;

    /// The result of a validation reduced to the facts the tests compare.
    #[derive(Debug, PartialEq, Eq)]
    enum Outcome {
        Secure,
        UntrustedOwner(String),
        UntrustedWriter(String),
        NullDacl,
    }

    fn outcome(result: Result<(), PathSecurityError>) -> Outcome {
        match result {
            Ok(()) => Outcome::Secure,
            Err(PathSecurityError::UntrustedOwner { principal, .. }) => Outcome::UntrustedOwner(principal),
            Err(PathSecurityError::UntrustedWriter { principal, .. }) => Outcome::UntrustedWriter(principal),
            Err(PathSecurityError::NullDacl { .. }) => Outcome::NullDacl,
            Err(other) => panic!("unexpected error {other:?}"),
        }
    }

    fn is_elevated() -> bool {
        get_security_context() == SecurityContext::Admin
    }

    fn current_sid() -> OwnedSid {
        current_user_sid(Path::new(".")).expect("current user SID")
    }

    fn current_sid_string() -> String {
        sid_to_string(current_sid().as_psid(), Path::new(".")).expect("SID string")
    }

    /// Indicates whether the current user is one of the principals every scope trusts, which
    /// happens when the tests run as SYSTEM.
    fn current_user_is_trusted() -> bool {
        let sid = current_sid();
        let psid = sid.as_psid();
        // SAFETY: psid points to a valid SID owned by sid.
        unsafe {
            IsWellKnownSid(psid, WinLocalSystemSid).as_bool()
                || IsWellKnownSid(psid, WinBuiltinAdministratorsSid).as_bool()
        }
    }

    fn set_security(path: &Path, information: OBJECT_SECURITY_INFORMATION, owner: Option<PSID>, dacl: Option<*const ACL>) {
        let wide = to_wide(path);
        // SAFETY: wide is a null terminated string and the owner and DACL pointers are valid for
        // the duration of the call.
        let status = unsafe {
            SetNamedSecurityInfoW(PCWSTR(wide.as_ptr()), SE_FILE_OBJECT, information, owner, None, dacl, None)
        };
        assert!(status.is_ok(), "SetNamedSecurityInfoW failed with error code {}", status.0);
    }

    /// Applies an SDDL string to a path as a protected DACL, and as the owner when the string
    /// defines one, so that nothing is inherited from the temp folder.
    fn apply_sddl(path: &Path, sddl: &str) {
        let text = HSTRING::from(sddl);
        let mut raw = PSECURITY_DESCRIPTOR(ptr::null_mut());
        // SAFETY: text is a null terminated string and raw receives a LocalAlloc descriptor owned
        // by SecurityDescriptor.
        unsafe { ConvertStringSecurityDescriptorToSecurityDescriptorW(PCWSTR(text.as_ptr()), SDDL_REVISION, &mut raw, None) }
            .expect("valid SDDL");
        let descriptor = SecurityDescriptor(raw);

        let mut defaulted = BOOL::default();
        let mut owner = PSID(ptr::null_mut());
        let mut present = BOOL::default();
        let mut dacl: *mut ACL = ptr::null_mut();
        // SAFETY: The descriptor is valid and the out pointers are writable locals.
        unsafe {
            GetSecurityDescriptorOwner(descriptor.0, &mut owner, &mut defaulted).expect("owner");
            GetSecurityDescriptorDacl(descriptor.0, &mut present, &mut dacl, &mut defaulted).expect("dacl");
        }

        let mut information = DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION;
        let owner = if owner.0.is_null() {
            None
        } else {
            information |= OWNER_SECURITY_INFORMATION;
            Some(owner)
        };
        let dacl = if present.as_bool() { Some(dacl.cast_const()) } else { None };
        set_security(path, information, owner, dacl);
    }

    fn apply_null_dacl(path: &Path) {
        set_security(path, DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION, None, None);
    }

    /// Builds a folder DACL with the trusted entries, full control for the current user and
    /// the given extra entries. The current user entry keeps the folder deletable when the tests
    /// are not elevated.
    fn user_folder_sddl(extra: &str) -> String {
        format!("D:P{TRUSTED}(A;OICI;FA;;;{}){extra}", current_sid_string())
    }

    /// Grants the current user full control again so a folder that a test locked down can be
    /// enumerated and removed. The owner always holds the right to change the DACL.
    fn restore_access(path: &Path) {
        apply_sddl(path, &user_folder_sddl(""));
    }

    fn folder_outcome(sddl: &str, scope: PathSecurityScope) -> Outcome {
        let dir = TempDir::new().expect("temp dir");
        apply_sddl(dir.path(), sddl);
        let result = validate_path(dir.path(), scope);
        restore_access(dir.path());
        dir.close().expect("remove temp dir");
        outcome(result)
    }

    #[test_case("" => Outcome::Secure; "trusted principals only")]
    #[test_case("(A;OICI;FR;;;WD)" => Outcome::Secure; "everyone read is allowed")]
    #[test_case("(A;OICI;FA;;;WD)" => Outcome::UntrustedWriter(EVERYONE.into()); "everyone full control")]
    #[test_case("(A;OICI;FA;;;BU)" => Outcome::UntrustedWriter(USERS.into()); "users full control")]
    #[test_case("(A;OICI;0x2;;;WD)" => Outcome::UntrustedWriter(EVERYONE.into()); "everyone write data")]
    #[test_case("(A;OICI;0x40000;;;WD)" => Outcome::UntrustedWriter(EVERYONE.into()); "everyone write dac")]
    #[test_case("(A;OICIIO;FA;;;WD)" => Outcome::Secure; "inherit only entries do not apply")]
    #[test_case("(D;OICI;FA;;;WD)" => Outcome::Secure; "deny entries do not grant")]
    #[test_case("(A;OICI;0x100;;;WD)" => Outcome::Secure; "write attributes alone is not write")]
    fn user_scope_folder(extra: &str) -> Outcome {
        folder_outcome(&user_folder_sddl(extra), PathSecurityScope::User)
    }

    #[test]
    fn machine_scope_requires_a_trusted_owner() {
        if is_elevated() {
            assert_eq!(folder_outcome(&format!("O:BAD:P{TRUSTED}"), PathSecurityScope::Machine), Outcome::Secure);
        } else if !current_user_is_trusted() {
            assert_eq!(
                folder_outcome(&format!("O:{}D:P{TRUSTED}", current_sid_string()), PathSecurityScope::Machine),
                Outcome::UntrustedOwner(current_sid_string())
            );
        }
    }

    #[test]
    fn machine_scope_rejects_write_access_for_the_current_user() {
        if !is_elevated() || current_user_is_trusted() {
            eprintln!("skipping: requires an elevated process running as a regular user");
            return;
        }
        let sid = current_sid_string();
        assert_eq!(
            folder_outcome(&format!("O:BAD:P{TRUSTED}(A;OICI;FA;;;{sid})"), PathSecurityScope::Machine),
            Outcome::UntrustedWriter(sid)
        );
    }

    #[test]
    fn current_user_owner_is_trusted_at_user_scope_only() {
        if current_user_is_trusted() {
            eprintln!("skipping: the current user is SYSTEM or Administrators");
            return;
        }
        let sid = current_sid_string();
        let sddl = format!("O:{sid}D:P{TRUSTED}");
        assert_eq!(folder_outcome(&sddl, PathSecurityScope::Machine), Outcome::UntrustedOwner(sid));
        assert_eq!(folder_outcome(&sddl, PathSecurityScope::User), Outcome::Secure);
    }

    #[test]
    fn null_dacl_is_insecure() {
        let dir = TempDir::new().expect("temp dir");
        apply_null_dacl(dir.path());
        let result = validate_path(dir.path(), PathSecurityScope::User);
        dir.close().expect("remove temp dir");
        assert_eq!(outcome(result), Outcome::NullDacl);
    }

    #[test]
    fn secure_file_in_secure_folder_is_secure() {
        let dir = TempDir::new().expect("temp dir");
        apply_sddl(dir.path(), &user_folder_sddl(""));
        let file = dir.path().join("policy.json");
        fs::write(&file, b"{}").expect("write file");
        let result = validate_path(&file, PathSecurityScope::User);
        dir.close().expect("remove temp dir");
        assert_eq!(outcome(result), Outcome::Secure);
    }

    #[test]
    fn insecure_folder_is_reported_before_the_file() {
        let dir = TempDir::new().expect("temp dir");
        apply_sddl(dir.path(), &user_folder_sddl("(A;OICI;FA;;;WD)"));
        let file = dir.path().join("policy.json");
        fs::write(&file, b"{}").expect("write file");
        let folder_path = path_string(dir.path());
        let result = validate_path(&file, PathSecurityScope::User);
        dir.close().expect("remove temp dir");
        assert_eq!(
            result,
            Err(PathSecurityError::UntrustedWriter {
                path: folder_path,
                principal: EVERYONE.into(),
            })
        );
    }

    #[test]
    fn insecure_file_is_reported_with_the_file_path() {
        let dir = TempDir::new().expect("temp dir");
        apply_sddl(dir.path(), &user_folder_sddl(""));
        let file = dir.path().join("policy.json");
        fs::write(&file, b"{}").expect("write file");
        apply_sddl(
            &file,
            &format!("D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;{})(A;;FA;;;WD)", current_sid_string()),
        );
        let result = validate_path(&file, PathSecurityScope::User);
        dir.close().expect("remove temp dir");
        assert_eq!(
            result,
            Err(PathSecurityError::UntrustedWriter {
                path: path_string(&file),
                principal: EVERYONE.into(),
            })
        );
    }

    /// Creates a temp folder whose children inherit a DACL that only trusts SYSTEM,
    /// Administrators and the current user.
    fn secure_root() -> TempDir {
        let dir = TempDir::new().expect("temp dir");
        apply_sddl(dir.path(), &user_folder_sddl(""));
        dir
    }

    /// Creates a folder below the root with the trusted entries, the current user and the given
    /// extra entries.
    fn child_folder(root: &TempDir, name: &str, extra: &str) -> PathBuf {
        let path = root.path().join(name);
        fs::create_dir(&path).expect("create folder");
        apply_sddl(&path, &user_folder_sddl(extra));
        path
    }

    /// Creates a file whose protected DACL has the trusted entries, the current user and the
    /// given extra entries.
    fn child_file(folder: &Path, name: &str, extra: &str) -> PathBuf {
        let path = folder.join(name);
        fs::write(&path, b"{}").expect("write file");
        apply_sddl(
            &path,
            &format!("D:P(A;;FA;;;SY)(A;;FA;;;BA)(A;;FA;;;{}){extra}", current_sid_string()),
        );
        path
    }

    /// Creates a junction, which unlike a symbolic link needs no privilege.
    fn create_junction(link: &Path, target: &Path) {
        let status = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .stdout(Stdio::null())
            .status()
            .expect("run mklink");
        assert!(status.success(), "mklink /J failed for {}", link.display());
    }

    /// Reports whether a symbolic link was created. Creating one needs a privilege that regular
    /// users only hold in developer mode, so the tests that need a symbolic link skip without it.
    fn linked(result: std::io::Result<()>) -> bool {
        match result {
            Ok(()) => true,
            Err(error) if error.raw_os_error() == Some(ERROR_PRIVILEGE_NOT_HELD) => {
                eprintln!("skipping: creating a symbolic link requires elevation or developer mode");
                false
            }
            Err(error) => panic!("create symbolic link: {error}"),
        }
    }

    #[test]
    fn junction_validates_the_folder_it_points_to() {
        let root = secure_root();
        let target = child_folder(&root, "target", EVERYONE_FULL);
        let link = root.path().join("link");
        create_junction(&link, &target);
        let link_alone = validate_object(&link, PathSecurityScope::User);
        let result = validate_path(&link, PathSecurityScope::User);
        root.close().expect("remove temp dir");
        assert_eq!(link_alone, Ok(()), "the junction itself only has trusted entries");
        assert_eq!(outcome(result), Outcome::UntrustedWriter(EVERYONE.into()));
    }

    #[test]
    fn junction_itself_must_be_secure() {
        let root = secure_root();
        let target = child_folder(&root, "target", "");
        let link = root.path().join("link");
        create_junction(&link, &target);
        apply_sddl(&link, &user_folder_sddl(EVERYONE_FULL));
        let target_alone = validate_path(&target, PathSecurityScope::User);
        let result = validate_path(&link, PathSecurityScope::User);
        root.close().expect("remove temp dir");
        assert_eq!(target_alone, Ok(()), "the folder the junction points to stays secure");
        assert_eq!(
            result,
            Err(PathSecurityError::UntrustedWriter {
                path: path_string(&link),
                principal: EVERYONE.into(),
            })
        );
    }

    #[test]
    fn symbolic_link_validates_the_folder_it_points_to() {
        let root = secure_root();
        let target = child_folder(&root, "target", EVERYONE_FULL);
        let link = root.path().join("link");
        if !linked(symlink_dir(&target, &link)) {
            return;
        }
        let result = validate_path(&link, PathSecurityScope::User);
        root.close().expect("remove temp dir");
        assert_eq!(outcome(result), Outcome::UntrustedWriter(EVERYONE.into()));
    }

    #[test]
    fn symbolic_link_validates_the_file_it_points_to() {
        let root = secure_root();
        let target_folder = child_folder(&root, "target", "");
        let file = child_file(&target_folder, "policy.json", "(A;;FA;;;WD)");
        let link_folder = child_folder(&root, "links", "");
        let link = link_folder.join("policy.json");
        if !linked(symlink_file(&file, &link)) {
            return;
        }
        let result = validate_path(&link, PathSecurityScope::User);
        root.close().expect("remove temp dir");
        assert_eq!(outcome(result), Outcome::UntrustedWriter(EVERYONE.into()));
    }

    #[test]
    fn intermediate_symbolic_link_validates_its_folder() {
        let root = secure_root();
        let target_folder = child_folder(&root, "target", "");
        let file = child_file(&target_folder, "policy.json", "");
        let hop_folder = child_folder(&root, "hop", EVERYONE_FULL);
        let hop = hop_folder.join("hop.json");
        let link_folder = child_folder(&root, "links", "");
        let link = link_folder.join("policy.json");
        if !linked(symlink_file(&file, &hop)) || !linked(symlink_file(&hop, &link)) {
            return;
        }
        let result = validate_path(&link, PathSecurityScope::User);
        root.close().expect("remove temp dir");
        match result {
            Err(PathSecurityError::UntrustedWriter { path, principal }) => {
                assert_eq!(principal, EVERYONE);
                assert!(Path::new(&path).ends_with("hop"), "expected the hop folder, got {path}");
            }
            other => panic!("expected an untrusted writer on the hop folder, got {other:?}"),
        }
    }

    #[test]
    fn chain_of_secure_links_is_secure() {
        let root = secure_root();
        let target_folder = child_folder(&root, "target", "");
        let file = child_file(&target_folder, "policy.json", "");
        // A relative link next to the file and an absolute link in another folder.
        let alias = target_folder.join("alias.json");
        let link_folder = child_folder(&root, "links", "");
        let link = link_folder.join("policy.json");
        if !linked(symlink_file("policy.json", &alias)) || !linked(symlink_file(&alias, &link)) {
            return;
        }
        let file_alone = validate_path(&file, PathSecurityScope::User);
        let result = validate_path(&link, PathSecurityScope::User);
        root.close().expect("remove temp dir");
        assert_eq!(file_alone, Ok(()));
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn link_chain_that_resolves_differently_is_insecure() {
        // Behind a folder symbolic link, Windows resolves the relative target of link.json
        // against the real folder. Joining the target to the path of the folder link instead
        // ends at the decoy file, which must not be accepted in place of the file that is read.
        let root = secure_root();
        let real = child_folder(&root, "real", "");
        child_file(root.path(), "policy.json", "");
        let outer = child_folder(&root, "outer", "");
        child_file(&outer, "policy.json", "");
        let folder_link = outer.join("inner");
        if !linked(symlink_dir(&real, &folder_link))
            || !linked(symlink_file(r"..\policy.json", real.join("link.json")))
        {
            return;
        }
        let through_link = folder_link.join("link.json");
        let result = validate_path(&through_link, PathSecurityScope::User);
        root.close().expect("remove temp dir");
        match result {
            Err(PathSecurityError::SecurityInfoRead { path, .. }) => {
                assert_eq!(path, path_string(&through_link));
            }
            other => panic!("expected the mismatch to be reported, got {other:?}"),
        }
    }

    #[test]
    fn current_user_sid_is_classified_by_context() {
        let sid = current_sid();
        let as_current = classify_sid(sid.as_psid(), Some(sid.as_psid()), Path::new(".")).expect("classify");
        let as_other = classify_sid(sid.as_psid(), None, Path::new(".")).expect("classify");
        assert!(as_current.id.starts_with("S-1-"), "unexpected SID {}", as_current.id);
        assert_eq!(as_current.id, as_other.id);
        if !current_user_is_trusted() {
            assert_eq!(as_current.kind, PrincipalKind::CurrentUser);
            assert_eq!(as_other.kind, PrincipalKind::Other);
        }
    }

    #[test]
    fn missing_path_is_reported() {
        let dir = TempDir::new().expect("temp dir");
        let missing = dir.path().join("missing");
        assert_eq!(
            validate_object(&missing, PathSecurityScope::User),
            Err(PathSecurityError::PathNotFound {
                path: path_string(&missing),
            })
        );
    }
}
