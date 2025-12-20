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

### Example 1 - Encode a script for transport

Encoding a PowerShell script as a data URI ensures safe transport through systems that may have
issues with special characters or line breaks.

```yaml
# dataUri.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  scriptContent:
    type: string
    defaultValue: "Write-Host 'Hello, World!'"
resources:
  - name: Encode script as data URI
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        originalScript: "[parameters('scriptContent')]"
        encodedScript: "[dataUri(parameters('scriptContent'))]"
```

```bash
dsc config get --file dataUri.example.1.dsc.config.yaml
```

```yaml
results:
- name: Encode script as data URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        originalScript: Write-Host 'Hello, World!'
        encodedScript: data:text/plain;charset=utf8;base64,V3JpdGUtSG9zdCAnSGVsbG8sIFdvcmxkISc=
messages: []
hadErrors: false
```

### Example 2 - Encode JSON configuration for embedding

The configuration encodes a JSON configuration string as a data URI, which is useful when passing
structured data through systems that expect URI-formatted content.

```yaml
# dataUri.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Encode JSON config as data URI
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUri('{\"setting\":\"value\",\"enabled\":true}')]"
```

```bash
dsc config get --file dataUri.example.2.dsc.config.yaml
```

```yaml
results:
- name: Encode JSON config as data URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: data:text/plain;charset=utf8;base64,eyJzZXR0aW5nIjoidmFsdWUiLCJlbmFibGVkIjp0cnVlfQ==
messages: []
hadErrors: false
```

### Example 3 - Compare base64 and dataUri encoding

Unlike the [`base64()`][02] function which returns only the encoded content, `dataUri()` adds
the data URI prefix with media type information. Use `dataUri()` when the target system expects
the full data URI format.

```yaml
# dataUri.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Compare encoding methods
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        base64Only: "[base64('Hello')]"
        fullDataUri: "[dataUri('Hello')]"
```

```bash
dsc config get --file dataUri.example.3.dsc.config.yaml
```

```yaml
results:
- name: Compare encoding methods
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        base64Only: SGVsbG8=
        fullDataUri: data:text/plain;charset=utf8;base64,SGVsbG8=
messages: []
hadErrors: false
```

### Example 4 - Encode multiline content

Multiline content like configuration files or scripts can be encoded as a data URI to preserve
line breaks during transport.

```yaml
# dataUri.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Encode multiline content
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUri('line1\nline2\nline3')]"
```

```bash
dsc config get --file dataUri.example.4.dsc.config.yaml
```

```yaml
results:
- name: Encode multiline content
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: data:text/plain;charset=utf8;base64,bGluZTEKbGluZTIKbGluZTM=
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
[02]: base64.md
