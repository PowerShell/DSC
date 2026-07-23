---
applyTo: "**/*.rs, **/*.ps1, **/*.psm1, **/*.tests.ps1, **/*.json"
---

# Code Review Instructions for DSC Repository

These instructions guide Copilot when performing code reviews on pull requests in this repository.
Focus on high-confidence, actionable findings. Do not comment on style, formatting, or trivial issues.

## General Principles

- **Do not flag issues that are intentional or already validated by CI**: If the code compiles and tests pass, do not claim it "will not compile" or "will fail." Verify your claim against actual Rust ownership/borrowing semantics before asserting a compile error.
- **Test resources vs production resources**: Code in `tools/dsctest/` is for testing only and is never user-facing. Do not apply production-quality error handling requirements (like replacing `expect()` with graceful errors) to test harnesses.
- **Automatically-generated files**: Files like `.versions.json` are updated by build automation. Do not flag version bumps in these files as unintentional unless the PR description explicitly contradicts them.

## Rust Code Patterns

### Caching and State Management

- **Cache key correctness**: When caching by resource type/version, verify the cache key accounts for all dimensions that affect the cached value. For adapter resources with a `target_resource`, the cache key must include the target resource identity, not just the adapter's type/version.
- **Duplicate caching**: Watch for cache writes that duplicate logic already handled elsewhere. `BTreeMap::extend` overwrites existing entries — if multiple versions of a resource should coexist in the cache, use entry-based insertion instead.
- **Unused imports after refactoring**: When cache writes or other logic are centralized, verify that moved-from modules no longer import the now-unused symbols.

### API Surface and Visibility

- **Minimize `pub` exposure**: Internal helpers, submodules, and filter functions used only within their parent module should be `pub(crate)` or private, not `pub`. Expanding the public API surface creates long-term maintenance burden.
- **Breaking changes to public APIs**: Adding required parameters to public functions is a breaking change. Consider backward-compatible alternatives (default parameters, builder pattern, or a new function name).
- **Prefer extracting shared logic**: When the same logic appears in multiple subcommands (e.g., table formatting, resource listing), extract it into a helper function rather than duplicating code. However, large refactors (like introducing new traits) should be separate PRs.

### Error Handling and Security

- **Fail closed for security checks**: Security-sensitive functions (ACL verification, permission checks) must treat failures as "not secure." If reading a security descriptor, enumerating ACEs, or calling stat fails, return the restrictive/denied result — never fail open.
- **NULL DACL detection**: A NULL DACL means full access to everyone. Security checks must treat `p_dacl.is_null()` as insecure.
- **Complete permission checks**: Write-access checks must cover `GENERIC_WRITE` and `GENERIC_ALL` in addition to specific write flags. Also consider inherit-only ACEs that don't apply to the object itself.
- **Error promotion**: Converting a previously-ignored error into a hard error (e.g., `join_paths` failure) can break existing environments. Prefer warnings or graceful degradation unless the error is truly unrecoverable.
- **Return `Option` for fallible lookups**: When a function can legitimately fail to find a path or value, return `Option<PathBuf>` (or similar) rather than an empty string, which can resolve to the current directory and cause confusing downstream behavior.

### Windows FFI and COM Safety

- **DLL loading security**: When loading system DLLs via `LoadLibraryW`, use `LoadLibraryExW` with `LOAD_LIBRARY_SEARCH_SYSTEM32` to prevent DLL preloading/hijacking attacks. This is especially important for resources that run with elevated privileges.
- **Resource cleanup with `Drop`**: COM objects, `VARIANT`s, and library handles (`HMODULE`) must be cleaned up even on error paths. Implement `Drop` for wrapper types or use RAII patterns to ensure `FreeLibrary`, `VariantClear`, etc. are always called.
- **Check HRESULT returns**: Windows API functions that return `HRESULT` (like `VariantClear`) should have their return values checked or at minimum logged, not silently ignored.
- **Iterative FFI operations**: When creating nested structures (e.g., registry keys path-by-path), ensure each iteration uses the result of the previous call as the parent handle, not always the root handle.

### Serialization and Consistency

- **Deterministic output**: If a function returns JSON, ensure the format (compact vs pretty) is consistent regardless of cache state. A cache hit should not produce differently-formatted output than a cache miss.
- **Avoid unnecessary serialize/parse roundtrips**: If you already have a `serde_json::Value`, cache or return it directly rather than serializing to string and re-parsing.
- **Schema/manifest version consistency**: When bumping a version in `Cargo.toml`, ensure the corresponding resource manifest `.dsc.resource.json` file is also updated to match.

### Concurrency

- **`ConcurrentQueue` draining**: Do not loop on `.IsEmpty` followed by `TryDequeue` — `IsEmpty` is only an approximation under concurrency. Use `while queue.TryDequeue(...)` as the single loop condition.

### What-If / Dry-Run Correctness

- **No side effects during what-if**: When implementing `--what-if` mode, verify that no code path before the what-if gate can mutate state. Functions like `ensure_config_exists()` that create/copy files must be gated or skipped in what-if mode.
- **Consistent platform enforcement in what-if**: If an operation is platform-restricted (e.g., Windows-only), what-if mode must enforce the same restriction. Do not allow what-if to succeed on unsupported platforms.

## PowerShell / Pester Test Patterns

### Cross-Platform Correctness

- **Path separators**: Never use hard-coded `\` in path construction. Always use `Join-Path` or `[System.IO.Path]::Combine()`. Tests with hard-coded backslashes will fail on Linux/macOS.
- **Platform-specific commands**: `stat -c` is GNU/Linux-specific. If a test should run on macOS as well, use PowerShell's `Get-Item` or gate the context on `$IsLinux` explicitly.
- **OS gating**: A Context labeled "Linux" that only checks `!$IsWindows` will also run on macOS. Be explicit with `$IsLinux` or `$IsMacOS`.

### Test Isolation and Cleanup

- **Preserve and restore environment variables**: When tests modify `$env:` variables (e.g., `DSC_RESTRICTED_PATH`, `DSC_RESOURCE_PATH`), always capture the original value in `BeforeAll` and restore it in `AfterAll`. Setting to `$null` unconditionally can destroy pre-existing values.
- **Conflicting environment variables**: If one env var takes precedence over another (e.g., `DSC_RESTRICTED_PATH` over `DSC_RESOURCE_PATH`), tests for the lower-priority var must explicitly clear the higher-priority one.
- **ACL and permission restoration**: When tests modify filesystem ACLs or permissions, capture the full original state and restore it completely. Do not rely on partial undo (e.g., removing a single ACE) or hard-coded permission values like `755`.
- **Recursive ACL changes**: If `icacls /T` is used to recursively modify ACLs, `AfterAll` must restore children as well, not just the top-level directory.
- **Event subscriber cleanup**: When using `Register-ObjectEvent`, clean up only the subscribers you created (filter by `-SourceIdentifier`), not all subscribers in the session via blanket `Get-EventSubscriber | Unregister-Event`.

### Test Assertions and Naming

- **Test name must match assertions**: If a test is named "X happens only once," the assertions must verify the "only once" constraint — not just that X happened at least once. Either tighten assertions or rename the test.
- **Ordering assumptions**: `dsc resource list` returns items in alphabetical order. Tests should document this assumption or sort results before asserting on specific positions.
- **Test logic correctness**: Verify that test conditions actually exercise the code path under test. A condition that always fails regardless of the variable being tested is a no-op test.
- **Assert `$LASTEXITCODE`**: When testing CLI commands, assert `$LASTEXITCODE -eq 0` (or the expected exit code) in addition to output checks. A non-zero exit can go unnoticed if only output content is validated.
- **Avoid duplicate reads**: When asserting on file content, read it once into a variable rather than calling `Get-Content` multiple times (once for the assertion, again for `-Because`).
- **Array comparisons**: `Should -Be` can be unreliable for array comparisons. Normalize arrays through JSON conversion before comparing, or compare individual elements.
- **Skip guards for cmdlet availability**: Tests that depend on platform-specific cmdlets (e.g., `Get-NetFirewallRule`) should check cmdlet availability in their `-Skip` condition, not just elevation status.

### Test Structure and Readability

- **Use Context blocks for shared setup**: When multiple test cases share setup steps (e.g., found vs. not-found scenarios), use a Pester `Context` block to group and clarify shared setup.
- **Separation between sections**: Maintain blank lines between logical sections in configuration and test files for readability.
- **Helpers belong near their usage**: Helper functions used only in a specific test file should live in that file. Only promote to the shared helper module (`build.helpers.psm1`) if used across multiple scripts.

## PowerShell Adapter Patterns

- **Module import can return arrays**: `Get-Module` can return multiple `PSModuleInfo` objects when multiple versions are loaded. Always select a single module (e.g., highest version) before calling methods on it.
- **Error handling around module imports**: When probing for DSC resource classes via module import (e.g., during cache refresh), wrap the import in try/catch so that one failing module doesn't abort the entire enumeration.
- **`$using:` in parallel blocks**: Variables from the parent scope are not automatically available inside `ForEach-Object -Parallel` script blocks. Use `$using:variableName` to capture them.
- **Distinguish terminating vs non-terminating errors**: Non-terminating errors written to the Error stream are counted in `$ps.HadErrors` but should not cause the adapter to report failure. Only terminating errors (exceptions) should produce a non-zero exit code.

## Design and Architecture Patterns

### Settings and Configuration Precedence

- **Full precedence chain**: Settings resolution must account for all scopes: Policy > CommandLine > Environment > Workspace > User > Install. Missing scopes in the chain can allow lower-precedence values to bypass higher-precedence ones.
- **Resolved fields vs containers**: When implementing settings resolution, each leaf setting should be individually resolvable (with its source scope tracked), not just the container as a whole.
- **`allow_override` enforcement**: When a setting's `allow_override` is false (policy-scoped), ensure the code path actually prevents override from env vars and CLI args — not just one of those sources.

### Backward Compatibility

- **Breaking test expectations**: If a change makes a previously-optional directive required, existing tests must still pass as-is. Behavior that differs from what tests already validate is a potential breaking change that needs working group discussion.
- **Resource filtering logic**: When engine-side filtering is added alongside resource-side filtering, the logic should be: (1) if the resource supports filtering, let it handle it; (2) if not, use engine filtering with an INFO-level message.
- **Semantic versioning for resources**: Resources below 1.0 (e.g., `0.x.y`) are not design-stable. For breaking changes to resource input/output shapes, bump the minor version. Reserve 1.x for stable designs.

### Regex and Wildcard Handling

- **Escape all metacharacters**: When converting user-supplied wildcard patterns to regex, escape all regex metacharacters (`[`, `(`, `+`, `^`, `$`, `|`, `\`, etc.), not just `.`. Unescaped metacharacters can cause panics or unexpected matching behavior.

### Resource Manifest and Schema Coherence

- **Schema must match implementation**: If a resource manifest's embedded schema declares a property as required, the resource implementation must enforce that requirement. Conversely, if the implementation requires a field, the schema's `required` array must include it.
- **Export schema validation**: When `export.schema` is defined, the engine uses it for input validation. Ensure the schema is strict enough that invalid inputs are caught before reaching the resource.
- **`noFiltering` semantics**: When a resource declares `export.schema: noFiltering`, the export input should be treated as empty (no filter properties passed to the resource).
- **Naming for DSC-specific conventions**: Use leading underscore (`_`) only for canonical (cross-resource) properties. Resource-specific properties that might collide with future keywords should use descriptive names (e.g., `sshd_config_filepath`) and be discussed in the working group if elevation to canonical status is warranted.

### CI/CD and GitHub Actions

- **Fork permission limitations**: `GITHUB_TOKEN` on fork PRs is typically read-only even with `pull-requests: write`. Steps that post PR comments should be gated to same-repo PRs or use `continue-on-error: true`.
- **Conditional tool installation**: Install expensive tools (Rust toolchains, coverage tools, protoc) only after determining the PR actually needs them (e.g., after checking which files changed).
- **PowerShell conventions in build scripts**: Functions should be singular per PS convention (e.g., `Test-RustProject` not `Test-RustProjects`). Build helper failures should throw so the pipeline stops on error.

## Documentation and Logging

- **Accurate comments**: If code behavior changes, update comments to match. A comment that says "retains quoting" when the code actually strips quotes is misleading.
- **Log level appropriateness**: Full `PATH` contents should be `trace!` level, not `debug!`. Sensitive or extremely verbose data should use the lowest practical log level.
- **Doc comments matching implementation**: If a doc comment describes behavior that the implementation doesn't actually enforce (e.g., "controls both env var and CLI option" when only env var is gated), update the comment to match reality.
- **Remove debug print statements**: `println!` debug output must not be left in production or test code. Use `debug!`/`trace!` macros from the tracing crate for diagnostics, or remove entirely before merge.
- **Locale/i18n string accuracy**: When adding localized strings, verify the key name matches the keyword/function it describes. Copy-paste errors in locale files (e.g., wrong keyword name in the error message) are common and hard to catch later.
- **Dead locale strings**: Do not add i18n keys that are never referenced in code. Unused locale strings accumulate and make it harder to identify which messages are active.
