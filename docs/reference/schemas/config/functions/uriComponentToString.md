---
description: Reference for the 'uriComponentToString' DSC configuration document function
ms.date: 10/10/2025
ms.topic: reference
title: uriComponentToString function
---

# uriComponentToString

## Synopsis

Returns a decoded string from a URI-encoded value.

## Syntax

```yaml
uriComponentToString(<uriEncodedString>)
```

## Description

The `uriComponentToString()` function decodes a URI-encoded string back to its original form.
It converts percent-encoded sequences (like `%20` for space or `%40` for `@`) back to their
original characters. This function is the inverse of [`uriComponent()`][01].

This function is useful when you need to decode URI components that were previously encoded,
such as query parameters, path segments, or other URI parts.

## Examples

### Example 1 - Decode a URI-encoded query parameter

This example decodes a URI-encoded query parameter value back to its original string.

```yaml
# example1.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo decoded value
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[uriComponentToString('John%20Doe')]"
```

```bash
dsc config get --document example1.dsc.yaml config get
```

```yaml
results:
- name: Echo decoded value
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: John Doe
```

### Example 2 - Decode a URI-encoded email address

This example decodes a URI-encoded email address with special characters.

```yaml
# example2.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo decoded email
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[uriComponentToString('user%2Btag%40example.com')]"
```

```bash
dsc config get --document example2.dsc.yaml config get
```

```yaml
results:
- name: Echo decoded email
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: user+tag@example.com
```

### Example 3 - Decode a complete URI-encoded URL

This example decodes a completely URI-encoded URL back to its readable form.

```yaml
# example3.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo decoded URL
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[uriComponentToString('https%3A%2F%2Fapi.example.com%2Fusers%3Fstatus%3Dactive')]"
```

```bash
dsc config get --document example3.dsc.yaml config get
```

```yaml
results:
- name: Echo decoded URL
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: https://api.example.com/users?status=active
```

### Example 4 - Round-trip encoding and decoding

This example demonstrates encoding a string with [`uriComponent()`][01] and then decoding it
back with `uriComponentToString()`, showing that they are inverse operations.

```yaml
# example4.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo round-trip result
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[uriComponentToString(uriComponent('Hello, World!'))]"
```

```bash
dsc config get --document example4.dsc.yaml config get
```

```yaml
results:
- name: Echo round-trip result
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello, World!
```

### Example 5 - Decode Unicode characters

This example decodes a URI-encoded string containing UTF-8 encoded Unicode characters.

```yaml
# example5.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo decoded Unicode
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[uriComponentToString('caf%C3%A9')]"
```

```bash
dsc config get --document example5.dsc.yaml config get
```

```yaml
results:
- name: Echo decoded Unicode
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: café
```

## Parameters

### uriEncodedString

The `uriComponentToString()` function expects a single string argument representing a
URI-encoded value. The function decodes any percent-encoded sequences (like `%20`, `%40`, etc.)
back to their original characters.

If the encoded string contains invalid percent-encoding sequences (such as incomplete sequences
or invalid hexadecimal digits), the function returns an error.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

The `uriComponentToString()` function returns the decoded string with all percent-encoded
sequences converted back to their original characters. The output is always a string.

```yaml
Type: string
```

## Related functions

The following functions are related to `uriComponentToString()`:

- [`uriComponent()`][01] - Encodes a string for safe use in URI components (inverse operation)
- [`uri()`][02] - Combines a base URI and relative URI with intelligent path handling
- [`base64ToString()`][03] - Decodes a base64-encoded string
- [`concat()`][04] - Combines multiple strings
- [`parameters()`][05] - Returns the value of a parameter

<!-- Link reference definitions -->
[01]: uriComponent.md
[02]: uri.md
[03]: base64ToString.md
[04]: concat.md
[05]: parameters.md
