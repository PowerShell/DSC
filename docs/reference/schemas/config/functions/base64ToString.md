---
description: Reference for the 'base64ToString' DSC configuration document function
ms.date:     09/30/2025
ms.topic:    reference
title:       base64ToString
---

# base64ToString

## Synopsis

Converts a base64 representation to a string.

## Syntax

```Syntax
base64ToString(<base64Value>)
```

## Description

The `base64ToString()` function converts a [base64][01] encoded string back to
its original string representation. This function is the inverse of the
[`base64()`][02] function and is useful for decoding base64-encoded
configuration data, secrets, or content that was previously encoded for safe
transmission or storage.## Examples

### Example 1 - Decode a base64 string

The configuration decodes a base64-encoded string back to its original value.

```yaml
# base64ToString.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Decode base64 string
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[base64ToString('aGVsbG8gd29ybGQ=')]"
```

```bash
dsc config get --file base64ToString.example.1.dsc.config.yaml
```

```yaml
results:
- name: Decode base64 string
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: hello world
messages: []
hadErrors: false
```

### Example 2 - Round-trip encoding and decoding

The configuration demonstrates encoding a string to base64 and then decoding it
back using the [`base64()`][02] function inside the `base64ToString()` function.

```yaml
# base64ToString.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Round-trip base64 conversion
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[base64ToString(base64('Configuration Data'))]"
```

```bash
dsc config get --file base64ToString.example.2.dsc.config.yaml 
```

```yaml
results:
- name: Round-trip base64 conversion
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Configuration Data
messages: []
hadErrors: false
```

### Example 3 - Decode configuration from parameters

This example shows decoding base64-encoded configuration data passed through
parameters, which is common when passing complex data through deployment
systems that require base64 encoding.

```yaml
# base64ToString.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  encodedConfig:
    type: string
    defaultValue: eyJzZXJ2ZXJOYW1lIjoid2ViLXNlcnZlci0wMSIsInBvcnQiOjgwODB9
resources:
  - name: Decode server configuration
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[base64ToString(parameters('encodedConfig'))]"
```

```bash
dsc config get --file base64ToString.example.3.dsc.config.yaml
```

```yaml
results:
- name: Decode server configuration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: '{"serverName":"web-server-01","port":8080}'
messages: []
hadErrors: false
```

### Example 4 - Decode with error handling

This example demonstrates how the function handles invalid base64 input by
using the [`if()`][03] function to provide fallback behavior.

```yaml
# base64ToString.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  possiblyEncodedData:
    type: string
    defaultValue: validBase64String=
  fallbackData:
    type: string
    defaultValue: default configuration
resources:
  - name: Safe decode with fallback
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        decodedValue: "[base64ToString(parameters('possiblyEncodedData'))]"
        fallback: "[parameters('fallbackData')]"
```

```bash
dsc --file base64ToString.example.4.dsc.config.yaml config get
```

```yaml
results:
- name: Safe decode with fallback
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        decodedValue: waEb(KidString
        fallback: default configuration
messages: []
hadErrors: false
```

## Parameters

### base64Value

The `base64ToString()` function expects a single string containing valid
base64-encoded data. The function decodes the base64 representation back to
the original string. If the value isn't a valid base64 string, DSC raises an
error. If the decoded bytes don't form valid UTF-8, DSC also raises an error.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `base64ToString()` function returns the decoded string representation of
the **base64Value** parameter.

```yaml
Type: string
```

## Exceptions

The `base64ToString()` function raises errors for the following conditions:

- **Invalid base64 encoding**: When the input string contains characters or
  patterns that are not valid base64
- **Invalid UTF-8**: When the decoded bytes do not form valid UTF-8 text

## Related functions

- [`base64()`][02] - Encodes a string to base64 format
- [`string()`][04] - Converts values to strings
- [`parameters()`][05] - Retrieves parameter values
- [`if()`][03] - Returns values based on a condition

<!-- Link reference definitions -->
[01]: https://en.wikipedia.org/wiki/Base64
[02]: ./base64.md
[03]: ./if.md
[04]: ./string.md
[05]: ./parameters.md
