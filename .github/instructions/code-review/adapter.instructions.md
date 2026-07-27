---
applyTo: 'adapters/**'
description: 'Code review guidance for DSC adapters (PowerShell, WMI, etc.)'
---

# Adapter Code Review

Adapters bridge DSC to external resource ecosystems (PowerShell DSC v1/v2 resources, WMI, etc.).
They run PowerShell runspaces, manage module loading, and translate between DSC and native formats.

## Module Import Safety

- **Module import can return arrays**: `Get-Module` can return multiple `PSModuleInfo` objects when multiple versions are loaded. Always select a single module (highest version) before calling methods.
- **Error handling around module imports**: Wrap import probes in try/catch so one failing module doesn't abort entire resource enumeration during cache refresh.
- **`$using:` in parallel blocks**: Variables from parent scope are not available inside `ForEach-Object -Parallel`. Use `$using:variableName`.

## Error Handling

- **Distinguish terminating vs non-terminating errors**: Non-terminating errors in `$ps.HadErrors` should not produce non-zero exit. Only terminating errors (exceptions) should cause adapter failure.
- **Use `$_` in catch blocks, not `$error`**: `$error` is the global error collection. Use `$_` (the caught exception) for condition checks.
- **Flush trace queues before exit on all paths**: If a catch/finally path exits the script, drain the trace queue first to avoid losing diagnostic messages.

## Concurrency and Event Handling

- **`ConcurrentQueue` draining**: Do not loop on `.IsEmpty` followed by `TryDequeue`. Use `while queue.TryDequeue(...)` as the single loop condition -- `IsEmpty` is only an approximation.
- **Event subscriber cleanup**: Filter by `-SourceIdentifier`, not blanket `Get-EventSubscriber | Unregister-Event`.

## Secrets and Security

- **Prevent command injection**: Flag code that embeds user-controlled values (secrets, vault names, paths) into PowerShell command strings. Prefer passing arguments/JSON.
- **Never leak secrets into logs**: Verify redaction is applied when logging resource output that may contain secure values.
