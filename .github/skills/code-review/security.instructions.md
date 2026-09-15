---
applyTo: 'lib/dsc-lib/src/util.rs,lib/dsc-lib-security_context/**,lib/dsc-lib-registry/**,resources/registry/**,resources/windows_firewall/**,resources/windows_service/**,resources/dism_dsc/**'
description: 'Code review guidance for security-sensitive code (ACLs, policy, credentials, elevated resources)'
---

# Security Code Review

This repo manages system configuration. Many resources run elevated and modify security-sensitive
state (ACLs, registry, services, firewall rules, policy files). Security review is critical.

## Fail Closed

- **All security checks must fail closed**: If reading a security descriptor, enumerating ACEs, or calling stat fails, return the restrictive/denied result -- never fail open.
- **NULL DACL detection**: A NULL DACL means full access to everyone. Treat `p_dacl.is_null()` as insecure.
- **Do not cache or trust failed verifications**: Trust caches (Authenticode, ACL checks) should only be updated on successful validation.

## ACL and Permission Checks

- **Complete write-access checks**: Cover `GENERIC_WRITE` and `GENERIC_ALL` in addition to specific write flags.
- **Inherit-only ACEs**: ACEs with `INHERIT_ONLY_ACE` flag don't apply to the object itself -- don't treat them as granting access to the folder.
- **Non-standard ACE variants**: Callback ACEs and object ACEs can still grant write access. Don't skip them.

## Policy Bypass Prevention

- **User inputs must not bypass policy**: CLI flags, env vars, or config changes must not override policy-enforced settings. Policy sources remain authoritative.
- **Enforce declared security context**: If manifests declare `requireSecurityContext`, verify runtime enforcement for every operation.

## DLL and Native Code Safety

- **DLL loading security**: Use `LoadLibraryExW` with `LOAD_LIBRARY_SEARCH_SYSTEM32` instead of `LoadLibraryW` to prevent DLL hijacking. Critical for elevated resources.
- **Resource cleanup with `Drop`**: COM objects, `VARIANT`s, and library handles must be cleaned up on all paths including error paths.
- **Check HRESULT returns**: Don't silently ignore Windows API return codes.

## Secrets

- **Never leak secrets into logs or output**: Verify redaction when logging output that may contain secure values.
- **Prevent command injection**: Don't embed user-controlled values into PowerShell command strings. Pass arguments/JSON instead.

## Destructive Operations

- **Unresolvable system objects**: In list-reconciling resources (firewall, services), skip system-created entries (AppX/UWP rules) that users cannot reliably reference.
- **Service credential changes**: Flag service-configuration code that switches accounts without secure credential handling.
