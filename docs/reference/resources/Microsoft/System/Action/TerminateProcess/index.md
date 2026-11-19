---
description: Microsoft.System.Action/TerminateProcess DSC action reference documentation
ms.date:     11/09/2026
ms.topic:    reference
title:       Microsoft.System.Action/TerminateProcess
---

# Microsoft.System.Action/TerminateProcess

## Synopsis

Terminates a process by PID or every process with an exact name on Windows, Linux, and macOS.

## Metadata

```yaml
Version    : 0.1.0
Kind       : action
Tags       : [Windows, Linux, macOS, Process]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: Terminate selected process
    type: Microsoft.System.Action/TerminateProcess
    properties:
      # Specify exactly one of pid or name.
      pid: 1234
```

## Description

This action runs only during configuration `set`. It forcefully terminates the selected process
using the native process library (`TerminateProcess` on Windows and `SIGKILL` on Unix).
It doesn't create processes or implement desired-state management.

Provide exactly one selector. `pid` targets one process; `name` targets every matching process
in a snapshot. Names match exactly, case-insensitively on Windows and case-sensitively on Linux
and macOS. Wildcards aren't accepted. Use the names returned by [Microsoft.System/Process][01],
including any platform-specific extension or truncation.

> [!WARNING]
> Termination can cause data loss. Specifying a name terminates **all** processes with that name
> that the caller has permission to terminate. Prefer a PID when targeting one instance.

Process 0 and the executable running the action can't be targeted. No matching processes, or a
failed termination request, produces an error. The action attempts every matching process even
if a termination fails. A partial failure can't be rolled back; the error lists failed PIDs.
Success means termination requests succeeded; process cleanup can finish asynchronously.
Processes started after enumeration aren't included.

The action returns a `terminated` array containing each targeted process's `pid`, `name`,
and `cmdline` snapshot. It doesn't support what-if.

> [!NOTE]
> This action is installed with DSC itself on all platforms. Update DSC to update this action.

## Requirements

The caller must have permission to terminate each target. Elevated privileges may be needed for
processes owned by other users. Protected processes can still reject termination.

## Capabilities

- `set` - Invokes termination through `dsc config set`.

## Examples

1. [Terminate a selected process][02]

## Properties

Exactly one of [pid](#pid) or [name](#name) is required.

### pid

<details><summary>Expand for <code>pid</code> property metadata</summary>

```yaml
Type       : integer
IsRequired : false
IsKey      : false
IsReadOnly : false
IsWriteOnly: false
Minimum    : 1
Maximum    : 4294967295
```

</details>

The process identifier to terminate. Can't be combined with `name`.

### name

<details><summary>Expand for <code>name</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : false
IsWriteOnly: false
```

</details>

An exact, nonempty process name. Can't contain `*` or be combined with `pid`.
Every process with this name is targeted.

## Instance validating schema

```json
{
  "type": "object",
  "additionalProperties": false,
  "oneOf": [
    { "required": ["pid"] },
    { "required": ["name"] }
  ],
  "properties": {
    "pid": { "type": "integer", "minimum": 1, "maximum": 4294967295 },
    "name": { "type": "string", "minLength": 1, "pattern": "^[^*]+$" }
  }
}
```

## Exit codes

- [0](#exit-code-0) - Success.
- [1](#exit-code-1) - Invalid arguments.
- [2](#exit-code-2) - Invalid input.
- [3](#exit-code-3) - Process enumeration or termination failed.

### Exit code 0

All targeted termination requests succeeded.

### Exit code 1

The executable arguments are invalid. An error message explains the expected syntax.

### Exit code 2

Input validation failed. An error message describes the invalid selector.

### Exit code 3

Enumeration failed, no process matched, the target was unsafe, or termination failed.
An error message describes the problem. Some processes might already have terminated.

## See also

- [Microsoft.System/Process][01]

<!-- Link definitions -->
[01]: ../../Process/index.md
[02]: ./examples/terminate-process.md
