---
description: Reference for the 'utcNow' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       utcNow
---

# utcNow

## Synopsis

Returns the current UTC timestamp when used as a parameter default.

## Syntax

```Syntax
utcNow()
utcNow(<format>)
```

## Description

The `utcNow()` function returns the current time in UTC. It can only be used
when defining the `defaultValue` of a parameter in a configuration document.
Using it elsewhere produces an error. When called without arguments, it returns
an ISO 8601 timestamp with microsecond precision. When a format string is
provided, the output uses that custom format.

The format string uses a subset of .NET date/time format patterns that DSC
internally maps to its formatting system. Unsupported tokens are passed
through literally.

## Examples

### Example 1 - Parameter default timestamp

The following example assigns the current UTC time as a default parameter
value, then echoes it.

```yaml
# utcnow.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  generatedAt:
    type: string
    defaultValue: "[utcNow()]"
resources:
- name: Show timestamp
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      generatedAt: "[parameters('generatedAt')]"
```

```bash
dsc config get --file utcnow.example.1.dsc.config.yaml
```

```yaml
results:
- name: Show timestamp
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        generatedAt: 2025-08-12T14:23:05.123456Z
messages: []
hadErrors: false
```

### Example 2 - Custom formatted timestamp

The following example uses a custom format string to produce a friendly date
stamp. The format maps .NET patterns (`yyyy-MM-dd HH:mm:ss`) to the internal
formatter.

```yaml
# utcnow.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  buildStamp:
    type: string
    defaultValue: "[utcNow('yyyy-MM-dd HH:mm:ss')]"
resources:
- name: Build metadata
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      buildStamp: "[parameters('buildStamp')]"
```

```bash
dsc config get --file utcnow.example.2.dsc.config.yaml
```

```yaml
results:
- name: Build metadata
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        buildStamp: 2025-08-12 14:23:05
messages: []
hadErrors: false
```

### Example 3 - Combine with other functions

The following example combines `utcNow()` with `concat()` and `string()` to
build an identifier that contains a timestamp.

```yaml
# utcnow.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  timestamp:
    type: string
    defaultValue: "[utcNow('yyyyMMdd-HHmmss')]"
resources:
- name: ID generator
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      releaseId: "[concat('release-', parameters('timestamp'))]"
```

```bash
dsc config get --file utcnow.example.3.dsc.config.yaml
```

```yaml
results:
- name: ID generator
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        releaseId: release-20250812-142305
messages: []
hadErrors: false
```

## Parameters

### format

An optional date/time format string using .NET-style tokens. If omitted, the
function returns an ISO 8601 UTC timestamp with microsecond precision.

```yaml
Type:     string
Required: false
```

## Output

The `utcNow()` function returns the current UTC timestamp as a string.

```yaml
Type: string
```

## Related functions

- [`string()`][00] - Converts values to strings
- [`concat()`][01] - Concatenates strings together
- [`uniqueString()`][02] - Produces a deterministic hash-based string

<!-- Link reference definitions -->
[00]: ./string.md
[01]: ./concat.md
[02]: ./uniqueString.md
