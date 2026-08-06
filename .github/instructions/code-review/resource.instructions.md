---
applyTo: 'resources/**'
description: 'Code review guidance for DSC resources (individual resource implementations)'
---

# Resource Code Review

Resources are individual components that manage specific system state (registry, services,
firewall, SSH config, DISM features, etc.). They run as separate executables invoked by the engine.

## Manifest and Schema Coherence

- **Schema must match implementation**: Required properties in schema must be enforced in code and vice versa.
- **Behavior must match manifest metadata**: Versions, `requireSecurityContext`, adapter args, and defaults must reflect what the resource actually does.
- **Schema/manifest version consistency**: When bumping a version in `Cargo.toml`, ensure the corresponding `.dsc.resource.json` manifest is also updated.
- **`noFiltering` semantics**: Export input should be treated as empty when `noFiltering` is declared.
- **Canonical property naming**: Leading underscore (`_`) is only for cross-resource canonical properties. Resource-specific properties use descriptive names (e.g., `sshd_config_filepath`).

## Operation Consistency

- **All operations must validate consistently**: If a parameter is validated in `get`, it should also be validated in `set`, `test`, and `export`.
- **What-if must not mutate state**: Verify no code path before the what-if gate can create files, modify config, or change system state.
- **Consistent platform enforcement in what-if**: Platform-restricted operations must enforce restrictions even in what-if mode.

## Error Handling

- **Prefer `Result` over panics**: Avoid `unwrap()`/`expect()` on user input or manifest data. Return clear error messages and stable exit codes.
- **`unwrap_or_default()` on serialization is a bug**: `serde_json::to_string(...).unwrap_or_default()` silently emits empty string on failure. Surface the error and exit non-zero.
- **Return `Option` for fallible lookups**: Return `Option<PathBuf>` rather than empty string (which resolves to `.`).

## Destructive Operations

- **Watch for unresolvable system objects**: In list-reconciling resources (firewall, services), flag logic that disables/removes system-created entries (AppX/UWP rules) that users cannot reliably reference in their declared state.
- **Semantic versioning**: Resources below 1.0 are not design-stable. Breaking changes to input/output shapes require a minor version bump.

## Regex and Wildcard Handling

- **Escape all metacharacters**: When converting wildcards to regex, escape `[`, `(`, `+`, `^`, `$`, `|`, `\` -- not just `.`. Unescaped metacharacters cause panics or unexpected matching.
