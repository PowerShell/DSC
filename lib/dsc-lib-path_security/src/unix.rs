// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Reads Unix ownership and mode bits into the security model.

use std::os::unix::fs::MetadataExt;
use std::path::Path;

use crate::model::{self, SecurityModel};
use crate::{PathSecurityError, PathSecurityScope, io_error};

/// Gets the effective user id of the current process.
///
/// # Returns
///
/// The effective user id.
fn effective_uid() -> u32 {
    nix::unistd::geteuid().as_raw()
}

/// Reads the owner and mode bits of a path into a security model.
///
/// A symbolic link at the end of the path is not followed, which matches the Windows
/// implementation. The caller resolves links before reading the object they point to.
///
/// # Arguments
///
/// 1. `path`: the file or folder to read.
///
/// # Errors
///
/// Returns [`PathSecurityError::PathNotFound`] when the path does not exist and
/// [`PathSecurityError::SecurityInfoRead`] when the metadata cannot be read.
fn read_security_model(path: &Path) -> Result<SecurityModel, PathSecurityError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| io_error(path, &error))?;
    Ok(model::unix::model_from_mode(
        path,
        metadata.uid(),
        metadata.gid(),
        metadata.mode(),
        effective_uid(),
    ))
}

/// Validates one existing file or folder without looking at its containing folder.
///
/// A symbolic link at the end of the path is not followed. The caller resolves links before
/// validating the object they point to.
///
/// # Arguments
///
/// 1. `path`: the absolute path of the file or folder.
/// 1. `scope`: the scope whose trust rules apply.
///
/// # Errors
///
/// Returns a [`PathSecurityError`] when the metadata cannot be read or when the owner or mode
/// bits violate the trust rules of the scope.
pub(crate) fn validate_object(path: &Path, scope: PathSecurityScope) -> Result<(), PathSecurityError> {
    let model = read_security_model(path)?;
    model::evaluate(&model, scope)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, Permissions};
    use std::os::unix::fs::{PermissionsExt, symlink};

    use tempfile::TempDir;

    use super::*;
    use crate::validate_path;

    fn is_root() -> bool {
        nix::unistd::geteuid().is_root()
    }

    fn set_mode(path: &Path, mode: u32) {
        fs::set_permissions(path, Permissions::from_mode(mode)).expect("set permissions");
    }

    fn path_of(path: &Path) -> String {
        path.display().to_string()
    }

    #[test]
    fn folder_owned_by_current_user_is_untrusted_at_machine_scope() {
        let dir = TempDir::new().expect("temp dir");
        let result = validate_path(dir.path(), PathSecurityScope::Machine);
        if is_root() {
            assert_eq!(result, Ok(()));
        } else {
            let expected = format!("uid {}", effective_uid());
            assert_eq!(
                result,
                Err(PathSecurityError::UntrustedOwner {
                    path: path_of(dir.path()),
                    principal: expected,
                })
            );
        }
    }

    #[test]
    fn folder_owned_by_current_user_is_trusted_at_user_scope() {
        let dir = TempDir::new().expect("temp dir");
        assert_eq!(validate_path(dir.path(), PathSecurityScope::User), Ok(()));
    }

    #[test]
    fn group_writable_folder_is_insecure() {
        let dir = TempDir::new().expect("temp dir");
        set_mode(dir.path(), 0o770);
        let metadata = fs::metadata(dir.path()).expect("metadata");
        assert_eq!(
            validate_path(dir.path(), PathSecurityScope::User),
            Err(PathSecurityError::UntrustedWriter {
                path: path_of(dir.path()),
                principal: format!("group {}", metadata.gid()),
            })
        );
    }

    #[test]
    fn world_writable_folder_is_insecure() {
        let dir = TempDir::new().expect("temp dir");
        set_mode(dir.path(), 0o707);
        assert_eq!(
            validate_path(dir.path(), PathSecurityScope::User),
            Err(PathSecurityError::UntrustedWriter {
                path: path_of(dir.path()),
                principal: "other users".into(),
            })
        );
    }

    #[test]
    fn secure_file_in_secure_folder_is_secure() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("policy.json");
        fs::write(&file, b"{}").expect("write file");
        set_mode(&file, 0o644);
        assert_eq!(validate_path(&file, PathSecurityScope::User), Ok(()));
    }

    #[test]
    fn insecure_file_is_reported_with_the_file_path() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("policy.json");
        fs::write(&file, b"{}").expect("write file");
        set_mode(&file, 0o666);
        let metadata = fs::metadata(&file).expect("metadata");
        assert_eq!(
            validate_path(&file, PathSecurityScope::User),
            Err(PathSecurityError::UntrustedWriter {
                path: path_of(&file),
                principal: format!("group {}", metadata.gid()),
            })
        );
    }

    #[test]
    fn insecure_folder_is_reported_before_the_file() {
        let dir = TempDir::new().expect("temp dir");
        let file = dir.path().join("policy.json");
        fs::write(&file, b"{}").expect("write file");
        set_mode(&file, 0o644);
        set_mode(dir.path(), 0o707);
        assert_eq!(
            validate_path(&file, PathSecurityScope::User),
            Err(PathSecurityError::UntrustedWriter {
                path: path_of(dir.path()),
                principal: "other users".into(),
            })
        );
    }

    #[test]
    fn symbolic_link_validates_the_folder_of_the_target() {
        let link_dir = TempDir::new().expect("temp dir for link");
        let target_dir = TempDir::new().expect("temp dir for target");
        let target = target_dir.path().join("policy.json");
        fs::write(&target, b"{}").expect("write file");
        set_mode(&target, 0o644);
        set_mode(target_dir.path(), 0o707);
        let link = link_dir.path().join("policy.json");
        symlink(&target, &link).expect("create symlink");

        assert_eq!(
            validate_path(&link, PathSecurityScope::User),
            Err(PathSecurityError::UntrustedWriter {
                path: path_of(target_dir.path()),
                principal: "other users".into(),
            })
        );
    }

    #[test]
    fn intermediate_symbolic_link_validates_its_folder() {
        let link_dir = TempDir::new().expect("temp dir for link");
        let hop_dir = TempDir::new().expect("temp dir for hop");
        let target_dir = TempDir::new().expect("temp dir for target");
        let target = target_dir.path().join("policy.json");
        fs::write(&target, b"{}").expect("write file");
        set_mode(&target, 0o644);
        let hop = hop_dir.path().join("hop.json");
        symlink(&target, &hop).expect("create hop symlink");
        let link = link_dir.path().join("policy.json");
        symlink(&hop, &link).expect("create symlink");
        set_mode(hop_dir.path(), 0o707);

        assert_eq!(
            validate_path(&link, PathSecurityScope::User),
            Err(PathSecurityError::UntrustedWriter {
                path: path_of(hop_dir.path()),
                principal: "other users".into(),
            })
        );
    }

    #[test]
    fn chain_of_secure_symbolic_links_is_secure() {
        let link_dir = TempDir::new().expect("temp dir for link");
        let target_dir = TempDir::new().expect("temp dir for target");
        let target = target_dir.path().join("policy.json");
        fs::write(&target, b"{}").expect("write file");
        set_mode(&target, 0o644);
        // A relative link in a subfolder of the target folder and an absolute link in another
        // folder.
        let sub_dir = target_dir.path().join("sub");
        fs::create_dir(&sub_dir).expect("create subfolder");
        set_mode(&sub_dir, 0o755);
        let alias = sub_dir.join("alias.json");
        symlink("../policy.json", &alias).expect("create relative symlink");
        let link = link_dir.path().join("policy.json");
        symlink(&alias, &link).expect("create symlink");

        assert_eq!(validate_path(&link, PathSecurityScope::User), Ok(()));
    }

    #[test]
    fn symbolic_link_to_a_folder_validates_the_target() {
        let link_dir = TempDir::new().expect("temp dir for link");
        let target_dir = TempDir::new().expect("temp dir for target");
        set_mode(target_dir.path(), 0o707);
        let link = link_dir.path().join("folder");
        symlink(target_dir.path(), &link).expect("create symlink");

        assert_eq!(
            validate_path(&link, PathSecurityScope::User),
            Err(PathSecurityError::UntrustedWriter {
                path: path_of(target_dir.path()),
                principal: "other users".into(),
            })
        );
    }

    #[test]
    fn symbolic_link_cycle_is_insecure() {
        let dir = TempDir::new().expect("temp dir");
        let first = dir.path().join("first");
        let second = dir.path().join("second");
        symlink(&second, &first).expect("create first symlink");
        symlink(&first, &second).expect("create second symlink");

        match validate_path(&first, PathSecurityScope::User) {
            Err(PathSecurityError::SecurityInfoRead { path, .. }) => assert_eq!(path, path_of(&first)),
            other => panic!("expected the cycle to be reported, got {other:?}"),
        }
    }

    #[test]
    fn device_is_not_a_file_or_folder() {
        let device = Path::new("/dev/null");
        if !device.exists() {
            return;
        }
        assert_eq!(
            validate_path(device, PathSecurityScope::User),
            Err(PathSecurityError::UnsupportedFileType {
                path: path_of(device),
            })
        );
    }

    #[test]
    fn missing_path_is_reported() {
        let dir = TempDir::new().expect("temp dir");
        let missing = dir.path().join("missing");
        assert_eq!(
            validate_object(&missing, PathSecurityScope::User),
            Err(PathSecurityError::PathNotFound {
                path: path_of(&missing),
            })
        );
    }
}
