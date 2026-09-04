---
description: JSON schema reference for the metadata field Microsoft.DSC
ms.date:     09/01/2026
ms.topic:    reference
title:       Microsoft.DSC metadata property schema reference
---

# Microsoft.DSC metadata property schema reference

## Synopsis

Metadata properties used and returned by DSC for configuration and resource operations.

## Description

The `Microsoft.DSC` metadata object captures execution details that DSC adds to command output and,
when applicable, to resource results. It describes what operation ran, when it started and
finished, how long it took, the security context DSC ran under, the DSC version that produced the
output, and any restarts that resources reported as required. These properties are informational
and help with diagnostics, auditing, and tooling.

Every property is optional. Which properties DSC includes depends on the context. The top-level
metadata for a configuration operation includes the operation, execution type, timestamps,
duration, security context, version, and any required restarts. The metadata for an individual
resource instance result includes only the duration of that instance's operation. Timestamps use
RFC 3339 `date-time` format, and durations use the ISO 8601 `duration` format.

Starting with DSC version 3.2.0, DSC returns the same execution information in the top-level
`executionInformation` property of command output and in the `executionInformation` property of
each resource instance result. The `metadata.Microsoft.DSC` property is retained for backwards
compatibility with tools and scripts that process DSC output. In DSC version 4.0.0, command output
will no longer include the `metadata.Microsoft.DSC` property. Prefer `executionInformation` when
writing new tools and scripts.

Consumers should tolerate additional, future metadata fields. Producers should preserve unknown
metadata they do not interpret.

## Properties

### duration

Defines the duration of a DSC operation against a configuration document or resource instance as a
string following the format defined in [ISO8601 ABNF for `duration`][01].

For example, `PT0.611216S` represents a duration of about `0.61` seconds.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/duration.json
Type:          string
Format:        duration
```

### endDatetime

Defines the end date and time for the DSC operation as a timestamp following the format defined in
[RFC3339, section 5.6 (see `date-time`)][02].

For example: `2024-04-14T08:49:51.395686600-07:00`

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/endDatetime.json
Type:          string
Format:        date-time
```

### executionType

Defines whether DSC actually applied an operation to the configuration or was run in what-if mode.
This property is always `actual` for `get`, `test`, and `export` operations. For `set` operations,
this value is `whatIf` when DSC is invoked with the `--what-if` argument.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/executionType.json
Type:          string
ValidValues:   [actual, whatIf]
```

### operation

Defines the operation that DSC applied to the configuration document: `get`, `set`, `test`, or
`export`.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/operation.json
Type:          string
ValidValues:   [get, set, test, export]
```

### restartRequired

Defines the list of restarts that resources reported as required after the operation. DSC collects
this information from the `_restartRequired` property that a resource returns in its result. The
top-level metadata for a configuration operation includes the entries reported by every instance in
the document. DSC only includes this property when at least one resource reported a required
restart.

Each item in the list is an object with exactly one of the following properties:

- `system` - A string that identifies the system that requires a restart.
- `service` - The name of a service that requires a restart.
- `process` - An object with the `name` (string) and `id` (integer) of a process that requires a
  restart.

Use the [restartRequired()][03] configuration function to check for required restarts in the
outputs of a configuration document.

```yaml
Type:            array
ItemsType:       object
ValidItemSchema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/executionInformation/restartRequired.json
```

### copyLoops

Defines the copy loop context for a resource instance that DSC expanded from a copy loop. When DSC
expands a copy loop, it adds this property to the metadata of each expanded instance. The value is
an object where each key is the name of a copy loop and the value is the zero-based iteration index
of the instance in that loop. DSC uses this information to resolve the `copyIndex()` function when
it evaluates the instance's properties. DSC doesn't include this property in command output.

```yaml
Type: object
```

### securityContext

Defines the security context that DSC was run under. If the value for this metadata property is
`elevated`, DSC was run as `root` (non-Windows) or an elevated session with Administrator
privileges (on Windows). If the value is `restricted`, DSC was run as a normal user or account in a
non-elevated session.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/securityContext.json
Type:          string
ValidValues:   [current, elevated, restricted]
```

### startDatetime

Defines the start date and time for the DSC operation as a timestamp following the format defined
in [RFC3339, section 5.6 (see `date-time`)][02].

For example: `2024-04-14T08:49:51.395686600-07:00`

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/startDatetime.json
Type:          string
Format:        date-time
```

### version

Defines the version of DSC that ran the command. This value is always the semantic version of the
DSC command, like `3.0.0-preview.7`.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/version.json
Type:          string
```

<!-- Reference link definitions -->
[01]: https://datatracker.ietf.org/doc/html/rfc3339#appendix-A
[02]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
[03]: ../../config/functions/restartRequired.md
