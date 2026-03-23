// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityContext {
    Admin,
    User,
}

#[cfg(target_os = "windows")]
#[must_use]
pub fn get_security_context() -> SecurityContext {
    use is_elevated::is_elevated;
    if is_elevated() {
        return SecurityContext::Admin;
    }
    SecurityContext::User
}

#[cfg(not(target_os = "windows"))]
#[must_use]
pub fn get_security_context() -> SecurityContext {
    use nix::unistd::Uid;

    if Uid::effective().is_root() {
        return SecurityContext::Admin;
    }
    SecurityContext::User
}
