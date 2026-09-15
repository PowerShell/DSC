---
applyTo: 'adapters/**'
description: 'Code review guidance for DSC adapters (PowerShell, WMI, etc.)'
---

# Adapter Code Review

Adapters bridge DSC to external resource ecosystems (PowerShell DSC v1/v2 resources, WMI providers,
etc.). An adapter is a single-mode executable that receives the path or content to the adapted
resource along with the type name and version information.

## General Adapter Design

- **Single-mode operation**: Adapters operate in a single mode -- they receive input describing
  which adapted resource to invoke and pass through the operation. Do not add multi-mode dispatch
  logic inside an adapter; each invocation handles exactly one resource call.
- **Input contract**: Adapters receive the resource type name, version information, and either a
  path to or the content of the adapted resource definition. Verify the adapter correctly parses
  and forwards all of these.
- **`validate` is deprecated**: The `validate` operation must NOT be implemented by adapters.
  Validation is handled by JSONSchema. Flag any new code that adds a `validate` operation or
  processes validate requests.
- **Schema-based validation**: Adapted resources should provide JSONSchema for input validation.
  The adapter should not re-implement validation logic that the schema already covers.
- **Exit codes**: Adapters must exit 0 on success and non-zero on failure. Ensure all error paths
  produce a non-zero exit code with structured error output on stderr.
- **Structured I/O**: Adapters communicate via JSON on stdin/stdout. Verify JSON serialization
  round-trips correctly, especially for nested objects, null values, and arrays.
- **Resource enumeration**: When listing available resources, adapters must accurately report
  type names, versions, and capabilities. Do not advertise capabilities the adapted resource
  does not support.

## Secrets and Security (All Adapters)

- **Prevent command injection**: Flag code that embeds user-controlled values (secrets, vault
  names, paths) into command strings. Prefer passing arguments or structured data.
- **Never leak secrets into logs**: Verify redaction is applied when logging resource output
  that may contain secure values.

## PowerShell Adapter Specifics

The PowerShell adapter runs PowerShell runspaces to invoke DSC v1/v2/v3 resources. These
patterns apply specifically to PowerShell-based adapter code.

### Module Import Safety

- **Module import can return arrays**: `Get-Module` can return multiple `PSModuleInfo` objects
  when multiple versions are loaded. Always select a single module (highest version) before
  calling methods.
- **Error handling around module imports**: Wrap import probes in try/catch so one failing module
  doesn't abort entire resource enumeration during cache refresh.
- **`$using:` in parallel blocks**: Variables from parent scope are not available inside
  `ForEach-Object -Parallel`. Use `$using:variableName`.

### Error Handling

- **Distinguish terminating vs non-terminating errors**: Non-terminating errors in `$ps.HadErrors`
  should not produce non-zero exit. Only terminating errors (exceptions) should cause adapter failure.
- **Use `$_` in catch blocks, not `$error`**: `$error` is the global error collection. Use `$_`
  (the caught exception) for condition checks.
- **Flush trace queues before exit on all paths**: If a catch/finally path exits the script, drain
  the trace queue first to avoid losing diagnostic messages.

### Concurrency and Event Handling

- **`ConcurrentQueue` draining**: Do not loop on `.IsEmpty` followed by `TryDequeue`. Use
  `while queue.TryDequeue(...)` as the single loop condition -- `IsEmpty` is only an approximation.
- **Event subscriber cleanup**: Filter by `-SourceIdentifier`, not blanket
  `Get-EventSubscriber | Unregister-Event`.
