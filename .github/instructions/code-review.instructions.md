---
applyTo: '**/*.rs,**/*.ps1,**/*.psm1,**/*.tests.ps1,**/*.json'
description: 'Repository-specific code review guidance for DSC (Rust, PowerShell/Pester, JSON)'
---

# Code Review Instructions for DSC Repository

These instructions guide Copilot when performing code reviews on pull requests in this repository.
Focus on high-confidence, actionable findings. Do not comment on style, formatting, or trivial issues.
Derived from analysis of 2,700+ review comments across 455 PRs in this repository.

## General Principles (False-Positive Avoidance)

- **Do not claim code will not compile unless you are certain**: Multiple reviews were rejected because Copilot incorrectly claimed Rust ownership/borrowing errors. If the code compiles and tests pass in CI, do not assert otherwise.
- **Test resources are not production code**: Code in `tools/dsctest/` is for testing only and is never user-facing. Do not require production-grade error handling (replacing `expect()` with `Result`) in test harnesses unless panics would hide regressions.
- **Automatically-generated files**: Files like `lib/dsc-lib-jsonschema/.versions.json` are updated by build automation. Do not flag version bumps as unintentional.
- **Do not demand large abstractions in small PRs**: If duplicated logic exists, suggest extracting a helper function. Do not block a focused PR by requesting a full trait/framework redesign -- that is follow-up work.
- **Table output is not a stable API**: JSON/YAML are the canonical machine-readable outputs. Do not frame table column/layout changes as breaking changes.
- **Separate input/output structs can be intentional**: Resources often distinguish desired state input from observed state output. Do not assume duplicate structs are accidental.
- **Intentional design decisions**: When maintainers explicitly label behavior as "intentional" (e.g., PATH mutation behavior, error promotion choices), do not re-flag. Respect the context of intentional design.

## Security

- **Fail closed on all security checks**: Security-sensitive functions (ACL verification, permission checks, policy folder validation) must treat failures as "not secure." If reading a security descriptor, enumerating ACEs, or calling stat fails, return the restrictive/denied result -- never fail open.
- **NULL DACL detection**: A NULL DACL means full access to everyone. Security checks must treat `p_dacl.is_null()` as insecure.
- **Complete permission checks**: Write-access checks must cover `GENERIC_WRITE` and `GENERIC_ALL` in addition to specific write flags. Also consider inherit-only ACEs (which don't apply to the object itself) and non-standard ACE variants (callback ACEs can still grant access).
- **Do not let user-controlled inputs bypass policy**: CLI flags, env vars, or config precedence changes must not override policy-enforced settings. Policy sources should remain authoritative even when user convenience switches are added.
- **DLL loading security**: When loading system DLLs via `LoadLibraryW`, use `LoadLibraryExW` with `LOAD_LIBRARY_SEARCH_SYSTEM32` to prevent DLL preloading/hijacking. Especially important for resources running elevated (DISM, offline registry, services).
- **Prevent command injection via string interpolation**: Flag code that embeds user-controlled values (secrets, vault names, paths) into PowerShell command strings. Prefer passing arguments/JSON rather than building quoted script fragments.
- **Never leak secrets into logs or output**: Verify redaction is consistently applied when logging JSON/resource output that may contain secure values. Review "show secrets" paths to ensure behavior is explicit.
- **Enforce declared security context at runtime**: If manifests declare `requireSecurityContext`, verify runtime enforcement exists for every operation (`get`, `set`, `test`, `export`). A schema field without enforcement is a security bug.
- **Do not cache or trust failed verifications**: Trust caches (e.g., Authenticode checks) should only be updated on successful validation, never on failure.
- **Watch for destructive modes with unresolvable system objects**: In list-reconciling resources (firewall, services), flag logic that disables/removes system-created entries (AppX/UWP rules) that users cannot reliably reference in their declared state.

## Rust Code Patterns

### Caching and Performance

- **Cache key correctness**: Cache keys must account for all dimensions affecting the cached value. For adapter resources with `target_resource`, cache by target identity, not just adapter type/version.
- **Centralize cache logic**: Flag duplicate cache writes or wrapper layers that both cache the same result. A single cache owner reduces redundant work and prevents divergent keying.
- **Avoid JSON serialize/parse roundtrips**: If you already have a `serde_json::Value`, cache or return it directly rather than stringifying and re-parsing.
- **Short-circuit redundant work**: Flag loops that keep scanning after all matches are found, or `contains_key` + `insert` patterns where entry APIs would do one pass.
- **Reduce allocations only when semantics stay intact**: Good flags: repeated `to_lowercase()`, cloning owned data that can be borrowed. Bad flags: clone removal that changes ownership semantics or breaks compilation. When in doubt, do not flag.

### API Surface and Visibility

- **Default to private/`pub(crate)`**: New modules, helpers, globals, and cache types should be private or `pub(crate)`. Require a clear external use case before exposing `pub`.
- **Breaking changes to public APIs**: Adding required parameters to public functions is a breaking change. Consider backward-compatible wrappers.
- **Extract shared logic into helpers**: When the same logic appears in multiple subcommands, extract a helper. But do not demand full trait redesigns in focused PRs.
- **Remove dead code immediately**: Unused imports, variables, and enums should be deleted when a refactor makes them unnecessary -- Rust warnings become CI failures.

### Error Handling

- **Prefer `Result` over panics for user input**: Avoid `unwrap()`/`expect()` on user input, manifest data, regex creation, or serialization. Return clear `Result` or stable exit codes. (Exception: test harness code in `tools/dsctest/`.)
- **Error promotion risk**: Converting previously-ignored errors into hard errors can break existing environments. Prefer warnings or graceful degradation unless truly unrecoverable.
- **Return `Option` for fallible lookups**: When a function can legitimately fail to find a path or value, return `Option<PathBuf>` rather than an empty string (which resolves to `.`).
- **`unwrap_or_default()` on serialization is a bug**: `serde_json::to_string(...).unwrap_or_default()` silently emits empty string on failure. Surface the error and exit non-zero.

### Windows FFI and COM Safety

- **Resource cleanup with `Drop`**: COM objects, `VARIANT`s, and library handles (`HMODULE`) must be cleaned up even on error paths. Implement `Drop` for wrapper types.
- **Check HRESULT returns**: Windows API functions that return `HRESULT` should have their return values checked or at minimum logged.
- **Iterative FFI operations**: When creating nested structures (registry keys path-by-path), ensure each iteration uses the previous call's result as parent handle, not always root.

### Consistency Across Operations

- **Behavior must be consistent across get/set/test/export**: If a validation, parameter, or output shape applies to one operation, verify it applies to all siblings.
- **Deterministic output**: JSON format (compact vs pretty) must be consistent regardless of cache state. A cache hit should not produce differently-formatted output than a miss.
- **Schema/manifest version consistency**: When bumping a version in `Cargo.toml`, ensure the corresponding `.dsc.resource.json` manifest is also updated.

### What-If / Dry-Run Correctness

- **No side effects during what-if**: Verify no code path before the what-if gate can mutate state. Functions like `ensure_config_exists()` that create/copy files must be gated.
- **Consistent platform enforcement**: If an operation is platform-restricted, what-if mode must enforce the same restriction.

### Naming and Serde Semantics

- **Names must match semantics**: Flag singular/plural mismatches, stale help text, misleading test names, and deprecated options still visibly advertised.
- **Be precise with `Option`/`Result`/serde**: Review whether `None`, empty collections, borrowed-vs-moved values, and `rename_all`/`deny_unknown_fields` all mean what the API claims.

### Concurrency

- **`ConcurrentQueue` draining**: Do not loop on `.IsEmpty` followed by `TryDequeue`. Use `while queue.TryDequeue(...)` as the single loop condition.

## PowerShell / Pester Test Patterns

### Cross-Platform Correctness

- **Path separators**: Never use hard-coded `\` in path construction. Always use `Join-Path`. Tests with hard-coded backslashes will fail on Linux/macOS.
- **Platform-specific commands**: `stat -c` is GNU/Linux-specific. Gate on `$IsLinux` explicitly or use PowerShell equivalents.
- **OS gating**: A Context that only checks `!$IsWindows` also runs on macOS. Be explicit with `$IsLinux` or `$IsMacOS`.
- **Use realistic cross-platform fixtures**: Test data should use correct path separators and plausible file locations for the target OS.

### Test Isolation and Cleanup

- **Preserve and restore environment variables**: Capture the original value in `BeforeAll` and restore in `AfterAll`. Never set to `$null` unconditionally.
- **Conflicting environment variables**: Tests for lower-priority env vars (e.g., `DSC_RESOURCE_PATH`) must explicitly clear higher-priority ones (`DSC_RESTRICTED_PATH`).
- **ACL and permission restoration**: Capture full original state and restore completely. Do not rely on partial undo or hard-coded permission values.
- **Recursive ACL changes**: If `icacls /T` is used, `AfterAll` must restore children too.
- **Event subscriber cleanup**: Filter by `-SourceIdentifier`, not blanket `Get-EventSubscriber | Unregister-Event`.
- **Use `-ErrorAction Ignore` over `SilentlyContinue`**: When leftover error records could pollute later assertions.

### Test Assertions and Naming

- **Test name must match assertions**: If named "X happens only once," assertions must verify the constraint, not just that X happened.
- **Assert `$LASTEXITCODE`**: Always check exit code in addition to output content for CLI command tests.
- **Prove the intended failure mode**: Negative tests must fail for the exact reason under test, not a broader fallback condition.
- **Cover both branches**: Add explicit tests for both "present" and "absent" cases rather than depending on host configuration.
- **Array comparisons**: Normalize arrays through JSON conversion before comparing with `Should -Be`.
- **Avoid duplicate reads**: Read file content once into a variable, not multiple times per assertion.
- **Skip guards for cmdlet availability**: Check cmdlet availability in `-Skip` conditions, not just elevation.
- **Ordering assumptions**: `dsc resource list` returns alphabetical order. Document or sort before position-based assertions.
- **Manifest file naming**: Discovery only loads manifests with `.dsc.resource.(json|yaml|yml)` suffix. Tests using other extensions won't exercise discovery.
- **Prefer `Should -BeExactly`**: Use exact comparisons when full expected result is known to prevent regressions slipping through loose checks.

### Test Structure

- **Use Context blocks for shared setup**: Group found/not-found scenarios with shared `BeforeEach`.
- **Helpers belong near their usage**: Only promote to `build.helpers.psm1` if used across multiple scripts.
- **Test both adapters**: If both `PowerShellScript` and `WindowsPowerShellScript` share code, parameterize tests to cover both.
- **Distinguish "failed" from "succeeded with errors"**: Add tests that separate terminating errors, non-terminating errors, and successful runs with warnings.
- **Add edge-case and malformed-input coverage**: Test "almost valid" negative cases and boundary conditions whenever only the happy path is covered.
- **Test behavior end-to-end**: Favor tests demonstrating user-visible failure/fix paths; use logs only when output alone cannot distinguish the bug.

## PowerShell Adapter Patterns

- **Module import can return arrays**: `Get-Module` can return multiple `PSModuleInfo` objects. Always select a single module (highest version) before calling methods.
- **Error handling around module imports**: Wrap import probes in try/catch so one failing module doesn't abort entire enumeration.
- **`$using:` in parallel blocks**: Variables from parent scope are not available inside `ForEach-Object -Parallel`. Use `$using:variableName`.
- **Distinguish terminating vs non-terminating errors**: Non-terminating errors in `$ps.HadErrors` should not produce non-zero exit. Only terminating errors (exceptions) should.
- **Use `$_` in catch blocks, not `$error`**: `$error` is the global error collection. Use `$_` (the caught exception) for condition checks in catch blocks.
- **Flush trace queues before exit on all paths**: If a catch/finally path exits the script, drain the trace queue first to avoid losing diagnostic messages.

## Design and Architecture Patterns

### Settings and Configuration Precedence

- **Full precedence chain**: Policy > CommandLine > Environment > Workspace > User > Install. Missing scopes allow bypasses.
- **Resolved fields vs containers**: Each leaf setting should be individually resolvable with source scope tracked.
- **`allow_override` enforcement**: When false (policy-scoped), must prevent override from both env vars and CLI args.

### Backward Compatibility

- **Breaking test expectations**: If existing tests must change, that signals a potential breaking change needing working group discussion.
- **Resource filtering logic**: (1) if resource supports filtering, let it handle it; (2) if not, use engine filtering with INFO message.
- **Semantic versioning**: Resources below 1.0 are not design-stable. Reserve 1.x for stable designs.
- **Public signature changes are compatibility events**: Adding parameters, changing flags, or exposing new modules should trigger compatibility review.

### Regex and Wildcard Handling

- **Escape all metacharacters**: When converting wildcards to regex, escape `[`, `(`, `+`, `^`, `$`, `|`, `\` -- not just `.`. Unescaped metacharacters cause panics or unexpected matching.

### Resource Manifest and Schema Coherence

- **Schema must match implementation**: Required properties in schema must be enforced in code and vice versa.
- **`noFiltering` semantics**: Export input should be treated as empty when declared.
- **Canonical property naming**: Leading underscore (`_`) is only for cross-resource canonical properties. Resource-specific properties use descriptive names (e.g., `sshd_config_filepath`).
- **Behavior must match manifest metadata**: Versions, `requireSecurityContext`, adapter args, and defaults must reflect what the resource actually does.

### CI/CD and GitHub Actions

- **Fork permission limitations**: Steps posting PR comments should gate on same-repo PRs or use `continue-on-error: true`.
- **Conditional tool installation**: Install expensive tools only after determining the PR needs them.
- **Check control flow, not just syntax**: Verify `if: always()`, `continue-on-error`, and `needs`/stage dependencies match stated failure behavior.
- **Preserve downstream artifact contracts**: Flag changes to artifact names/paths when downstream jobs expect the old layout.
- **PowerShell conventions**: Singular function names (`Test-RustProject`). Build failures should throw.

## Documentation and Logging

- **Accurate comments**: If code behavior changes, update comments to match.
- **Log level appropriateness**: Full `PATH` contents at `trace!`, not `debug!`. Never log secrets.
- **Doc comments matching implementation**: Update doc comments when described behavior doesn't match reality.
- **Remove debug print statements**: No `println!` debug output in production or test code. Use `debug!`/`trace!` macros.
- **Locale/i18n string accuracy**: Verify key names match the keyword/function they describe. Copy-paste errors in locale files are common.
- **Dead locale strings**: Do not add i18n keys never referenced in code.
- **Prefer scan-friendly wording**: Log/error strings should be concise and punctuated clearly for readability in diagnostics.
