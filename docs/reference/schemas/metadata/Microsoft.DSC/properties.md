---
description: JSON schema reference for the metadata field Microsoft.DSC
ms.date:     07/03/2025
ms.topic:    reference
title:       Microsoft.DSC metadata property schema reference
---

# Microsoft.DSC metadata property schema reference

## Synopsis

Metadata properties used and returned by DSC for configuration and resource operations.

## Description

Blah

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

### endDateTime

Defines the end date and time for the DSC operation as a timestamp following the format defined in
[RFC3339, section 5.6 (see `date-time`)][02].

For example: `2024-04-14T08:49:51.395686600-07:00`

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/endDateTime.json
Type:          string
Format:        date-time
```

### executionType

Defines whether DSC actually applied an operation to the configuration or was run in `WhatIf` mode.
This property is always `Actual` for `Get`, `Test`, and `Export` operations. For `Set` operations,
this value is `WhatIf` when DSC is invoked with the `--whatIf` argument.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/executionType.json
Type:          string
ValidValues:  [Actual, WhatIf]
```

### operation

Defines the operation that DSC applied to the configuration document: `Get`, `Set`, `Test`, or
`Export`.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/operation.json
Type:          string
ValidValues:  [Get, Set, Test, Export]
```

### securityContext

Defines the security context that DSC was run under. If the value for this metadata property is
`Elevated`, DSC was run as `root` (non-Windows) or an elevated session with Administrator
privileges (on Windows). If the value is `Restricted`, DSC was run as a normal user or account in a
non-elevated session.

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/metadata/Microsoft.DSC/securityContext.json
Type:          string
ValidValues:  [Current, Elevated, Restricted]
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
Type:          object
```

<!-- Reference link definitions -->
[01]: https://datatracker.ietf.org/doc/html/rfc3339#appendix-A
[02]: https://datatracker.ietf.org/doc/html/rfc3339#section-5.6
