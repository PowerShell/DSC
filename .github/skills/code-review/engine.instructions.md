---
applyTo: 'lib/dsc-lib/src/configure/**,lib/dsc-lib/src/discovery/**,lib/dsc-lib/src/dscresources/**,lib/dsc-lib/src/functions/**,lib/dsc-lib/src/settings/**'
description: 'Code review guidance for DSC engine (configure, discovery, resource dispatch, functions, settings)'
---

# Engine Code Review

The engine is the core of DSC: configuration processing, resource discovery, function evaluation,
schema caching, and settings resolution. Changes here have broad impact.

## Caching

- **Cache key correctness**: Cache keys must account for all dimensions affecting the cached value. For adapter resources with `target_resource`, cache by target identity, not just adapter type/version.
- **Centralize cache logic**: Flag duplicate cache writes or wrapper layers that both cache the same result. A single cache owner reduces redundant work and prevents divergent keying.
- **Avoid JSON serialize/parse roundtrips**: If you already have a `serde_json::Value`, cache or return it directly rather than stringifying and re-parsing.

## Settings Precedence

- **Full precedence chain**: Policy > CommandLine > Environment > Workspace > User > Install. Missing scopes allow bypasses.
- **Resolved fields vs containers**: Each leaf setting should be individually resolvable with source scope tracked.
- **`allow_override` enforcement**: When false (policy-scoped), must prevent override from both env vars and CLI args.

## Discovery

- **Short-circuit redundant work**: Flag loops that keep scanning after all matches are found, or `contains_key` + `insert` patterns where entry APIs would do one pass.
- **Error promotion risk**: Converting previously-ignored errors into hard errors (e.g., `join_paths` failure) can break existing environments. Prefer warnings or graceful degradation unless truly unrecoverable.

## Resource Dispatch Consistency

- **Behavior must be consistent across get/set/test/export**: If a validation, parameter, or output shape applies to one operation, verify it applies to all siblings.
- **Deterministic output**: JSON format (compact vs pretty) must be consistent regardless of cache state.
- **Resource filtering logic**: (1) if resource supports filtering, let it handle it; (2) if not, use engine filtering with INFO message.

## What-If / Dry-Run

- **No side effects during what-if**: Verify no code path before the what-if gate can mutate state. Functions like `ensure_config_exists()` that create/copy files must be gated.
- **Consistent platform enforcement**: If an operation is platform-restricted, what-if mode must enforce the same restriction.

## API Surface

- **Default to private/`pub(crate)`**: New modules, helpers, globals, and cache types should be private or `pub(crate)`. Require a clear external use case before exposing `pub`.
- **Breaking changes to public APIs**: Adding required parameters to public functions is a breaking change. Consider backward-compatible wrappers.
- **Public signature changes are compatibility events**: Adding parameters, changing flags, or exposing new modules should trigger compatibility review.
