---
description: Microsoft.System/Process resource reference documentation
ms.date:     11/09/2026
ms.topic:    reference
title:       Microsoft.System/Process
---

# Microsoft.System/Process

## Synopsis

Retrieves information about running processes on Windows, Linux, and macOS.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows, Linux, macOS, Process]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: Processes
    type: Microsoft.System/Process
    properties:
      # For get, specify pid or an exact, unambiguous name.
      # All properties are optional export filters.
      pid: 1234
      name: pwsh*
```

## Description

This read-only resource retrieves one process with `get` and enumerates processes with `export`.
It replaces the experimental `Microsoft/Process` resource. It doesn't echo input for `get`, `set`,
or `test`, and export is no longer limited to four processes.

For `get`, specify a PID or an exact name. No match is an error. Multiple matches are also an
error; use a PID to select one instance or export to retrieve all matches. Wildcards are accepted
only by export.

Export accepts optional filters for `pid`, `name`, and `cmdline`. All supplied filters must match.
The `name` filter supports `*`, matching zero or more characters, including multiple wildcards.
Other characters, including `?` and brackets, are literal. Names are case-insensitive on Windows
and case-sensitive on Linux and macOS. Omitted filters export all processes visible to the caller.
No matching processes is a successful export with no instances.

Process information is a snapshot: processes can exit or start immediately afterward.
Names are those reported by the operating system, including extensions on Windows and possible
platform-specific truncation on Unix. Command-line arguments may be unavailable because of
permissions or because the process has exited; in that case, `cmdline` is an empty array.
Unlike the predecessor, `cmdline` is an array of individual arguments, not a debug-formatted string.

This resource uses DSC's synthetic test functionality. It doesn't have the `set` capability and
can't modify system state. To terminate processes, use [TerminateProcess][01].

> [!NOTE]
> This resource is installed with DSC itself on all platforms. Update DSC to update this resource.

## Capabilities

- `get` - Retrieves the actual state of one process.
- `export` - Retrieves the actual state of all matching processes.

For more information, see [DSC resource capabilities][02].

## Examples

1. [Query and export running processes][03]

## Properties

There are no universally required properties. For `get`, at least one of `pid` or `name` is required.
The key property is [pid](#pid). [name](#name) is an optional selector and export filter.
[cmdline](#cmdline) is read-only and can also be supplied as an exact export filter.

### pid

<details><summary>Expand for <code>pid</code> property metadata</summary>

```yaml
Type       : integer
IsRequired : false
IsKey      : true
IsReadOnly : false
IsWriteOnly: false
Minimum    : 0
Maximum    : 4294967295
```

</details>

The numeric process identifier. Export compares it exactly; it isn't a wildcard.

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

The operating-system process name, not an executable path. Must not be empty.
For export only, `*` matches zero or more characters.

### cmdline

<details><summary>Expand for <code>cmdline</code> property metadata</summary>

```yaml
Type       : array
IsRequired : false
IsKey      : false
IsReadOnly : true
IsWriteOnly: false
```

</details>

An array of strings containing the command-line arguments reported by the operating system.
Filtering compares the entire array exactly, without wildcard expansion.

## Instance validating schema

```json
{
  "type": "object",
  "properties": {
    "pid": { "type": "integer", "minimum": 0, "maximum": 4294967295 },
    "name": { "type": "string", "minLength": 1 },
    "cmdline": { "type": "array", "items": { "type": "string" }, "readOnly": true }
  },
  "additionalProperties": false
}
```

## Exit codes

- [0](#exit-code-0) - Success.
- [1](#exit-code-1) - Invalid arguments.
- [2](#exit-code-2) - Invalid input.
- [3](#exit-code-3) - Process query failed.

### Exit code 0

The query completed successfully, including exports without matches.

### Exit code 1

The executable arguments are invalid. An error message explains the expected syntax.

### Exit code 2

The input JSON or selectors are invalid. An error message identifies the invalid input.

### Exit code 3

Process enumeration failed, or get matched zero or multiple processes. An error message explains
the failure; specify a PID when a name is ambiguous.

## See also

- [Microsoft.System.Action/TerminateProcess][01]
- [DSC resource capabilities][02]

<!-- Link definitions -->
[01]: ../Action/TerminateProcess/index.md
[02]: ../../../../../concepts/resources/capabilities.md
[03]: ./examples/query-processes.md
