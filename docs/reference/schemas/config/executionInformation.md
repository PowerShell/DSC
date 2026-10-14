---
description: JSON schema reference for the execution information DSC returns for configuration operations.
ms.date:     10/13/2026
ms.topic:    reference
title:       DSC execution information schema reference
---

# DSC execution information schema reference

## Synopsis

Describes the DSC operation that produced a result or a configuration document.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/config/executionInformation.json
Type:          object
```

## Description

Starting with DSC version 3.2.0, DSC describes the operation it performed in the
`executionInformation` property. DSC returns this property in the following places:

- The top level of the command output for the `dsc config get`, `dsc config set`,
  `dsc config test`, and `dsc config export` commands, where it describes the overall operation.
- Each item in the `results` property of the command output, where it describes the operation for
  that resource instance.
- The configuration document that the `dsc config export` command returns, and each resource
  instance in that document.

The schemas for a [configuration document][01] and a [resource instance][02] accept this property
as input, but DSC ignores it when it processes a configuration document.

The execution information replaces the `Microsoft.DSC` property of the `metadata` object, which
DSC still returns for backwards compatibility with tools and scripts that process DSC output. In
DSC version 4.0.0, command output will no longer include the `metadata.Microsoft.DSC` property.
Prefer `executionInformation` when writing new tools and scripts. For more information, see
[Microsoft.DSC metadata property schema reference][03].

Every property is optional. Which properties DSC includes depends on the context. The top-level
execution information for a configuration operation includes the operation, execution type,
timestamps, duration, security context, version, and any required restarts. The execution
information for an individual resource instance result includes only the duration of that
instance's operation and any restart the instance reported as required. Timestamps use RFC 3339
`date-time` format, and durations use the ISO 8601 `duration` format.

Consumers should tolerate additional, future properties. Producers should preserve unknown
properties that they don't interpret.

## Properties

### copyLoops

Defines the copy loop context for a resource instance that DSC expanded from a copy loop. When DSC
expands a copy loop, it adds this property to the execution information of each expanded instance.
The value is an object where each key is the name of a copy loop and the value is the zero-based
iteration index of the instance in that loop. DSC uses this information to resolve the
[copyIndex()][04] function when it evaluates the instance's properties. DSC doesn't include this
property in command output.

```yaml
Type: object
```

### duration

Defines the duration of a DSC operation against a configuration document or resource instance as a
string following the format defined in [ISO8601 ABNF for `duration`][05].

For example, `PT0.611216S` represents a duration of about `0.61` seconds.

```yaml
Type:   string
Format: duration
```

### endDatetime

Defines the end date and time for the DSC operation as a timestamp following the format defined in
[RFC3339, section 5.6 (see `date-time`)][06].

For example: `2024-04-14T08:49:51.395686600-07:00`

```yaml
Type:   string
Format: date-time
```

### executionType

Defines whether DSC actually applied an operation to the configuration or was run in what-if mode.
This property is always `actual` for `get`, `test`, and `export` operations. For `set` operations,
this value is `whatIf` when DSC is invoked with the `--what-if` argument.

```yaml
Type:        string
ValidValues: [actual, whatIf]
```

### operation

Defines the operation that DSC applied to the configuration document: `get`, `set`, `test`, or
`export`.

```yaml
Type:        string
ValidValues: [get, set, test, export]
```

### restartRequired

Defines the list of restarts that resources reported as required after the operation. DSC collects
this information from the `_restartRequired` property that a resource returns in its result. The
top-level execution information for a configuration operation includes the entries reported by
every instance in the document. DSC only includes this property when at least one resource
reported a required restart.

Each item in the list is an object with exactly one of the following properties:

- `system` - A string that identifies the system that requires a restart.
- `service` - The name of a service that requires a restart.
- `process` - An object with the `name` (string) and `id` (integer) of a process that requires a
  restart.

Use the [restartRequired()][07] configuration function to check for required restarts in the
outputs of a configuration document.

```yaml
Type:            array
ItemsType:       object
ValidItemSchema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/executionInformation/restartRequired.json
```

### securityContext

Defines the security context that DSC was run under. If the value for this property is
`elevated`, DSC was run as `root` (non-Windows) or an elevated session with Administrator
privileges (on Windows). If the value is `restricted`, DSC was run as a normal user or account in a
non-elevated session.

```yaml
Type:        string
ValidValues: [current, elevated, restricted]
```

### startDatetime

Defines the start date and time for the DSC operation as a timestamp following the format defined
in [RFC3339, section 5.6 (see `date-time`)][06].

For example: `2024-04-14T08:49:51.395686600-07:00`

```yaml
Type:   string
Format: date-time
```

### version

Defines the version of DSC that ran the command. This value is always the semantic version of the
DSC command, like `3.0.0-preview.7`.

```yaml
Type: string
```

### whatIf

Describes the what-if operation that a resource reported when DSC invoked it with the `--what-if`
argument. DSC only includes this property in the execution information for a resource instance
result when the execution type is `whatIf` and the resource returned what-if information for the
operation. The value is the object that the resource returned.

```yaml
Type: object
```

<!-- Link reference definitions -->
[01]: document.md#executioninformation
[02]: resource.md#executioninformation
[03]: ../metadata/Microsoft.DSC/properties.md
[04]: functions/copyIndex.md
[05]: https://datatracker.ietf.org/doc/html/rfc3339#appendix-A
[06]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
[07]: functions/restartRequired.md
