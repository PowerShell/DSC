---
description: Reference for the 'substring' DSC configuration document function
ms.date:     09/27/2025
ms.topic:    reference
title:       substring
---

# substring

## Synopsis

Returns a substring that starts at the specified character position and contains
the specified number of characters.

## Syntax

```Syntax
substring(<stringToParse>, <startIndex>)
substring(<stringToParse>, <startIndex>, <length>)
```

## Description

The `substring()` function extracts a portion of a string based on the specified
starting position and optional length. The function uses zero-based indexing,
meaning the first character is at position 0. This is useful for parsing
identifiers, extracting prefixes or suffixes, manipulating configuration values,
or formatting display strings.

Key behaviors:

- **Zero-based indexing**: The first character is at index 0
- **Optional length**: If length is omitted, returns the remainder of the string
  from the start position
- **Boundary validation**: Prevents access beyond string boundaries with clear
  error messages

## Examples

### Example 1 - Extract environment from resource name

This example demonstrates extracting environment information from standardized
resource names for conditional configuration. It uses the [`parameters()`][08]
function to retrieve the resource name.

```yaml
# substring.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  resourceName:
    type: string
    defaultValue: svc-api-prod-east
resources:
- name: Extract environment
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      environment: "[substring(parameters('resourceName'), 8, 4)]"
```

```bash
dsc config get --file substring.example.1.dsc.config.yaml
```

```yaml
results:
- name: Extract environment
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        environment: prod
messages: []
hadErrors: false
```

### Example 2 - Extract region from resource identifier

This example shows extracting a region code from a standardized resource
identifier without specifying length, using [`parameters()`][08] to retrieve
the identifier.

```yaml
# substring.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  resourceId:
    type: string
    defaultValue: app-web-eastus2-001
resources:
- name: Extract region
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      region: "[substring(parameters('resourceId'), 8)]"
```

```bash
dsc config get --file substring.example.2.dsc.config.yaml
```

```yaml
results:
- name: Extract region
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        region: eastus2-001
messages: []
hadErrors: false
```

### Example 3 - Parse version components

This example demonstrates parsing semantic version strings to extract major,
minor, and patch components. It uses [`parameters()`][08] to retrieve the
version string.

```yaml
# substring.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  version:
    type: string
    defaultValue: "3.2.1"
resources:
- name: Parse version
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      major: "[substring(parameters('version'), 0, 1)]"
      minor: "[substring(parameters('version'), 2, 1)]"
      patch: "[substring(parameters('version'), 4, 1)]"
```

```bash
dsc config get --file substring.example.3.dsc.config.yaml
```

```yaml
results:
- name: Parse version
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        major: "3"
        minor: "2"
        patch: "1"
messages: []
hadErrors: false
```

### Example 4 - Unicode and emoji support

This example shows that `substring()` correctly handles Unicode characters and
emojis. It uses [`parameters()`][08] to retrieve the message string.

```yaml
# substring.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  message:
    type: string
    defaultValue: "Hello 🌍 World!"
resources:
- name: Unicode substring
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      greeting: "[substring(parameters('message'), 0, 5)]"
      emoji: "[substring(parameters('message'), 6, 1)]"
      remainder: "[substring(parameters('message'), 8)]"
```

```bash
dsc config get --file substring.example.4.dsc.config.yaml
```

```yaml
results:
- name: Unicode substring
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        greeting: Hello
        emoji: 🌍
        remainder: " World!"
messages: []
hadErrors: false
```

## Parameters

### stringToParse

The original string from which the substring is extracted.

```yaml
Type:     string
Required: true
Position: 1
```

### startIndex

The zero-based starting character position for the substring. Must be a
non-negative integer and cannot exceed the length of the string.

```yaml
Type:     int
Required: true
Position: 2
```

### length

The number of characters for the substring. Must be a non-negative integer. The
start index plus length cannot exceed the length of the string. If omitted, the
remainder of the string from the start position is returned.

```yaml
Type:     int
Required: false
Position: 3
```

## Output

The `substring()` function returns a string containing the extracted portion of
the original string.

```yaml
Type: string
```

## Exceptions

The `substring()` function raises errors for the following conditions:

- **Invalid start index**: When `startIndex` is negative
- **Start index out of bounds**: When `startIndex` exceeds the string length
- **Invalid length**: When `length` is negative
- **Length out of bounds**: When `startIndex + length` exceeds the string
  length

## Related functions

- [`string()`][00] - Converts values to strings
- [`concat()`][01] - Concatenates strings together
- [`indexOf()`][02] - Finds the index of a substring in a string
- [`lastIndexOf()`][03] - Finds the last index of a substring in a string
- [`length()`][04] - Returns the length of a string or array
- [`startsWith()`][05] - Checks if a string starts with a prefix
- [`endsWith()`][06] - Checks if a string ends with a suffix

<!-- Link reference definitions -->
[00]: ./string.md
[01]: ./concat.md
[02]: ./indexOf.md
[03]: ./lastIndexOf.md
[04]: ./length.md
[05]: ./startsWith.md
[06]: ./endsWith.md
[08]: ./parameters.md
