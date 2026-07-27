---
applyTo: '**/*.tests.ps1,**/tests/**,**/test/**,**/Tests/**'
description: 'Code review guidance for DSC tests (Pester and Rust integration tests)'
---

# Test Code Review

Tests use Pester 5 for end-to-end CLI testing and Rust's built-in test framework for unit tests.
Tests run cross-platform (Windows, Linux, macOS) in CI.

## Cross-Platform Correctness

- **Path separators**: Never use hard-coded `\` in path construction. Always use `Join-Path`. Tests with hard-coded backslashes will fail on Linux/macOS.
- **Platform-specific commands**: `stat -c` is GNU/Linux-specific. Gate on `$IsLinux` explicitly or use PowerShell equivalents.
- **OS gating**: A Context that only checks `!$IsWindows` also runs on macOS. Be explicit with `$IsLinux` or `$IsMacOS`.
- **Use realistic cross-platform fixtures**: Test data should use correct path separators and plausible file locations for the target OS.

## Test Isolation and Cleanup

- **Preserve and restore environment variables**: Capture the original value in `BeforeAll` and restore in `AfterAll`. Never set to `$null` unconditionally.
- **Conflicting environment variables**: Tests for lower-priority env vars (e.g., `DSC_RESOURCE_PATH`) must explicitly clear higher-priority ones (`DSC_RESTRICTED_PATH`).
- **ACL and permission restoration**: Capture full original state and restore completely. Do not rely on partial undo or hard-coded permission values.
- **Recursive ACL changes**: If `icacls /T` is used, `AfterAll` must restore children too.
- **Event subscriber cleanup**: Filter by `-SourceIdentifier`, not blanket unregister.
- **Use `-ErrorAction Ignore` over `SilentlyContinue`**: When leftover error records could pollute later assertions.

## Assertions

- **Test name must match assertions**: If named "X happens only once," assertions must verify the constraint.
- **Assert `$LASTEXITCODE`**: Always check exit code in addition to output content for CLI tests.
- **Prove the intended failure mode**: Negative tests must fail for the exact reason under test, not a broader fallback.
- **Cover both branches**: Test both "present" and "absent" cases explicitly.
- **Array comparisons**: Normalize arrays through JSON conversion before comparing with `Should -Be`.
- **Avoid duplicate reads**: Read file content once into a variable, not multiple times per assertion.
- **Skip guards for cmdlet availability**: Check cmdlet availability in `-Skip`, not just elevation.
- **Ordering assumptions**: `dsc resource list` returns alphabetical order. Sort before position-based assertions.
- **Manifest file naming**: Discovery only loads `.dsc.resource.(json|yaml|yml)` files. Tests with other extensions won't exercise discovery.
- **Prefer `Should -BeExactly`**: Use exact comparisons when full expected result is known.

## Structure

- **Use Context blocks for shared setup**: Group found/not-found scenarios with shared `BeforeEach`.
- **Helpers belong near their usage**: Only promote to `build.helpers.psm1` if used across multiple scripts.
- **Test both adapters**: If `PowerShellScript` and `WindowsPowerShellScript` share code, parameterize tests.
- **Distinguish "failed" from "succeeded with errors"**: Separate terminating errors, non-terminating errors, and successful runs with warnings.
- **Add edge-case and malformed-input coverage**: Test boundary conditions whenever only the happy path is covered.
- **Test behavior end-to-end**: Favor tests demonstrating user-visible paths.
