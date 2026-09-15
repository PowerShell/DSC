---
applyTo: 'resources/apt/**,resources/sshdconfig/**,lib/dsc-lib-pal/**,resources/brew/**'
description: 'Code review guidance for Linux/macOS-specific code (SSH, apt, platform abstraction)'
---

# Linux/macOS Code Review

Linux and macOS resources manage SSH configuration, package managers, and platform-specific state.
Cross-platform correctness is a recurring theme since the same test suite runs on all platforms.

## Cross-Platform Path Handling

- **Never hard-code path separators**: Use platform-appropriate path joining. Rust's `PathBuf`/`Path::join` handles this automatically.
- **Platform-conditional logic**: Gate Linux-specific behavior (like `/etc/dsc` permissions) on the correct platform check. Don't assume "not Windows" means "Linux" -- it could be macOS.

## SSH Config Resource

- **Quoting preservation**: Understand what the parser actually preserves vs strips. Comments claiming "retains quoting" when code strips quotes are misleading.
- **Match block handling**: Ensure match/criteria blocks are included in export results and not accidentally filtered out.
- **Repeatable keywords with operators**: Keywords with `+`/`-`/`^` operators have specific semantics about repetition. Operators on non-repeatable keywords are invalid.

## Permission Checks (Linux)

- **Fail closed on stat failures**: If `fs::metadata` fails for `/etc/dsc`, do not proceed as if the folder is trusted.
- **Correct permission checks**: Use PowerShell's `Get-Item` or explicit `$IsLinux` gating rather than GNU-specific `stat -c` (which fails on macOS).

## Package Managers

- **Idempotent operations**: Package install/remove operations should be safe to run repeatedly without error.
- **Version comparison**: OS and package versions don't follow semver. Don't assume semver parsing applies.
