---
applyTo: 'resources/windows_firewall/**,resources/windows_service/**,resources/WindowsUpdate/**,resources/windows_personalization/**,resources/dism_dsc/**,resources/registry/**,lib/dsc-lib-registry/**,lib/dsc-lib-pal/**'
description: 'Code review guidance for Windows-specific code (FFI, COM, registry, DISM, services)'
---

# Windows Code Review

Windows-specific code uses Win32 APIs, COM interfaces, registry access, DISM, and Windows
service management. These resources typically run elevated.

## FFI Safety

- **DLL loading security**: Use `LoadLibraryExW` with `LOAD_LIBRARY_SEARCH_SYSTEM32` to prevent DLL hijacking.
- **Resource cleanup with `Drop`**: COM objects, `VARIANT`s, and `HMODULE` handles must be cleaned up on all paths. Implement `Drop` for wrapper types.
- **Check HRESULT returns**: Don't silently ignore Windows API error codes.
- **Iterative handle operations**: When creating nested structures (registry keys), use previous call's result as parent handle, not always root.

## ACL and Permissions

- **Fail closed**: Security descriptor read failures must return "not secure."
- **NULL DACL = full access**: Always treat as insecure.
- **Complete write mask**: Check `GENERIC_WRITE`, `GENERIC_ALL`, and all relevant specific flags.
- **Inherit-only ACEs**: Don't treat `INHERIT_ONLY_ACE` as applying to the folder itself.
- **Callback/object ACEs**: Non-standard ACE types can still grant access. Don't skip them.

## Service and Firewall Resources

- **Unresolvable system objects**: AppX/UWP firewall rules with names like "ms-resource://" cannot be referenced by users. Skip them in destructive reconciliation.
- **Service credential handling**: Flag logon identity changes without secure credential supply.
- **Protocol normalization**: Handle `Option<protocol>` correctly -- protocol may be required for the operation even if optional in the input struct.

## COM Enumeration

- **`VariantClear` on all paths**: Clear variants even when `.cast()` or other operations fail early via `?`.
- **Single-item vs collection**: COM enumerations may return collections. Handle both cases.
