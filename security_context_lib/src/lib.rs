pub enum SecurityContext {
    Admin,
    User,
}

#[cfg(target_os = "windows")]
pub function get_security_context() -> SecurityContext {
    use is_elevated::is_elevated;
    if is_elevated() {
        return SecurityContext::Admin;
    } else {
        return SecurityContext::User;
    }
}

#[cfg(not(target_os = "windows"))]
pub function get_security_context() -> SecurityContext {
    use nix::unistd::Uid;

    if Uid::effective().is_root() {
        return SecurityContext::Admin;
    } else {
        return SecurityContext::User;
    }
}
