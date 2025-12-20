---
description: Reference for the 'dataUri' DSC configuration document function
ms.date:     12/20/2025
ms.topic:    reference
title:       dataUri
---

# dataUri

## Synopsis

Converts a value to a data URI.

## Syntax

```Syntax
dataUri(<stringToConvert>)
```

## Description

The `dataUri()` function converts a string value to a [data URI][01] format. The function encodes
the input string as base64 and returns it as a data URI with the `text/plain` media type and
`utf8` charset.

Data URIs are useful for embedding small text content directly in configuration documents,
especially when the content needs to be passed through systems that expect URI-formatted data.

## Examples

### Example 1 - Convert a string to data URI

The configuration converts a basic string value with the `dataUri()` function.

```yaml
# dataUri.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Convert 'Hello' to data URI
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUri('Hello')]"
```

```bash
dsc config get --file dataUri.example.1.dsc.config.yaml
```

```yaml
results:
- name: Convert 'Hello' to data URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: data:text/plain;charset=utf8;base64,SGVsbG8=
messages: []
hadErrors: false
```

### Example 2 - Convert a concatenated string to data URI

The configuration uses the [concat()][02] function inside the `dataUri()` function to combine
strings before converting to a data URI.

```yaml
# dataUri.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Convert concatenated string to data URI
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUri(concat('Hello', ', World!'))]"
```

```bash
dsc config get --file dataUri.example.2.dsc.config.yaml
```

```yaml
results:
- name: Convert concatenated string to data URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: data:text/plain;charset=utf8;base64,SGVsbG8sIFdvcmxkIQ==
messages: []
hadErrors: false
```

### Example 3 - Round-trip encoding and decoding

The configuration demonstrates encoding a string to a data URI and then decoding it back using the
[dataUriToString()][03] function.

```yaml
# dataUri.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Round-trip data URI conversion
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUriToString(dataUri('Configuration Data'))]"
```

```bash
dsc config get --file dataUri.example.3.dsc.config.yaml
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

## Parameters

### stringToConvert

The `dataUri()` function expects a single string as input. The function converts the value into a
data URI representation. If the value isn't a string, DSC raises an error when validating the
configuration document.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `dataUri()` function returns a data URI string in the format
`data:text/plain;charset=utf8;base64,<encoded-content>` where `<encoded-content>` is the base64
representation of the **stringToConvert** value.

```yaml
Type: string
```

<!-- Link reference definitions -->
[01]: https://en.wikipedia.org/wiki/Data_URI_scheme
[02]: concat.md
[03]: dataUriToString.md
