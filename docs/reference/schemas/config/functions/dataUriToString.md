---
description: Reference for the 'dataUriToString' DSC configuration document function
ms.date:     12/20/2025
ms.topic:    reference
title:       dataUriToString
---

# dataUriToString

## Synopsis

Converts a data URI formatted value to a string.

## Syntax

```Syntax
dataUriToString(<dataUriToConvert>)
```

## Description

The `dataUriToString()` function converts a [data URI][01] formatted value back to its original
string representation. This function is the inverse of the [`dataUri()`][02] function and is useful
for decoding data that was previously encoded as a data URI.

The function supports both base64-encoded data URIs (those containing `;base64` in the metadata)
and URL-encoded data URIs. It automatically detects the encoding method and decodes accordingly.

## Examples

### Example 1 - Decode a base64-encoded data URI

The configuration decodes a data URI back to its original string value.

```yaml
# dataUriToString.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Decode data URI
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUriToString('data:text/plain;charset=utf8;base64,SGVsbG8=')]"
```

```bash
dsc config get --file dataUriToString.example.1.dsc.config.yaml
```

```yaml
results:
- name: Decode data URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello
messages: []
hadErrors: false
```

### Example 2 - Decode a data URI from parameter

This example shows decoding a data URI value passed through a parameter.

```yaml
# dataUriToString.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  dataFormattedString:
    type: string
    defaultValue: "data:;base64,SGVsbG8sIFdvcmxkIQ=="
resources:
  - name: Decode data URI from parameter
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUriToString(parameters('dataFormattedString'))]"
```

```bash
dsc config get --file dataUriToString.example.2.dsc.config.yaml
```

```yaml
results:
- name: Decode data URI from parameter
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello, World!
messages: []
hadErrors: false
```

### Example 3 - Round-trip encoding and decoding

The configuration demonstrates encoding a string to a data URI and then decoding it
back using the [`dataUri()`][02] function inside the `dataUriToString()` function.

```yaml
# dataUriToString.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Round-trip data URI conversion
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUriToString(dataUri('Configuration Data'))]"
```

```bash
dsc config get --file dataUriToString.example.3.dsc.config.yaml
```

```yaml
results:
- name: Round-trip data URI conversion
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Configuration Data
messages: []
hadErrors: false
```

### Example 4 - Decode URL-encoded data URI

This example demonstrates decoding a data URI that uses URL encoding instead of base64.

```yaml
# dataUriToString.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Decode URL-encoded data URI
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUriToString('data:text/plain,Hello%20World')]"
```

```bash
dsc config get --file dataUriToString.example.4.dsc.config.yaml
```

```yaml
results:
- name: Decode URL-encoded data URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello World
messages: []
hadErrors: false
```

## Parameters

### dataUriToConvert

The `dataUriToString()` function expects a single string containing a valid data URI. The data URI
must start with `data:` and contain a comma separating the metadata from the encoded data. If the
metadata contains `;base64`, the data is decoded as base64; otherwise, it's decoded as URL-encoded.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `dataUriToString()` function returns the decoded string representation of the
**dataUriToConvert** parameter.

```yaml
Type: string
```

## Exceptions

The `dataUriToString()` function raises errors for the following conditions:

- **Invalid data URI format**: When the input string doesn't start with `data:` or doesn't contain
  a comma separator
- **Invalid base64 encoding**: When the data portion contains invalid base64 characters (for
  base64-encoded data URIs)
- **Invalid UTF-8**: When the decoded bytes do not form valid UTF-8 text

## Related functions

- [`dataUri()`][02] - Converts a value to a data URI
- [`base64()`][03] - Encodes a string to base64 format
- [`base64ToString()`][04] - Decodes a base64 string
- [`parameters()`][05] - Retrieves parameter values

<!-- Link reference definitions -->
[01]: https://en.wikipedia.org/wiki/Data_URI_scheme
[02]: ./dataUri.md
[03]: ./base64.md
[04]: ./base64ToString.md
[05]: ./parameters.md
