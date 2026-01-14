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

The function supports only base64-encoded data URIs (those containing `;base64` in the metadata).
Non-base64 or URL-encoded data URIs aren't supported and result in an error.

## Examples

### Example 1 - Decode embedded script content

Decoding a PowerShell script from a data URI is useful when receiving commands from external
systems that transmit data in this format.

```yaml
# dataUriToString.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  encodedScript:
    type: string
    defaultValue: "data:text/plain;charset=utf8;base64,V3JpdGUtSG9zdCAnSGVsbG8sIFdvcmxkISc="
resources:
  - name: Decode and display script
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUriToString(parameters('encodedScript'))]"
```

```bash
dsc config get --file dataUriToString.example.1.dsc.config.yaml
```

```yaml
results:
- name: Decode and display script
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Write-Host 'Hello, World!'
messages: []
hadErrors: false
```

### Example 2 - Extract JSON configuration from data URI

The configuration decodes a JSON configuration that was transmitted as a data URI, then parses it
using the [`json()`][06] function to access its properties.

```yaml
# dataUriToString.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  configDataUri:
    type: string
    defaultValue: "data:;base64,eyJzZXR0aW5nIjoidmFsdWUiLCJlbmFibGVkIjp0cnVlfQ=="
resources:
  - name: Decode and parse JSON config
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        rawJson: "[dataUriToString(parameters('configDataUri'))]"
        parsedSetting: "[json(dataUriToString(parameters('configDataUri'))).setting]"
```

```bash
dsc config get --file dataUriToString.example.2.dsc.config.yaml
```

```yaml
results:
- name: Decode and parse JSON config
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        rawJson: '{"setting":"value","enabled":true}'
        parsedSetting: value
messages: []
hadErrors: false
```

### Example 3 - Process data from Azure ARM template output

Azure ARM templates and similar systems often use data URIs for content encoding. Use
`dataUriToString()` to decode this content back to its original form.

```yaml
# dataUriToString.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  armTemplateOutput:
    type: string
    defaultValue: "data:text/plain;charset=utf8;base64,SGVsbG8sIFdvcmxkIQ=="
resources:
  - name: Process ARM template data URI output
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[dataUriToString(parameters('armTemplateOutput'))]"
```

```bash
dsc config get --file dataUriToString.example.3.dsc.config.yaml
```

```yaml
results:
- name: Process ARM template data URI output
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello, World!
messages: []
hadErrors: false
```

### Example 4 - Round-trip verification

Encoding a string to a data URI and decoding it back verifies that data survives the
transformation correctly. Combine `dataUriToString()` with [`dataUri()`][02] to test this.

```yaml
# dataUriToString.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  originalContent:
    type: string
    defaultValue: "Configuration with special chars: <>&\""
resources:
  - name: Verify round-trip encoding
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        original: "[parameters('originalContent')]"
        afterRoundTrip: "[dataUriToString(dataUri(parameters('originalContent')))]"
```

```bash
dsc config get --file dataUriToString.example.4.dsc.config.yaml
```

```yaml
results:
- name: Verify round-trip encoding
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        original: 'Configuration with special chars: <>&"'
        afterRoundTrip: 'Configuration with special chars: <>&"'
messages: []
hadErrors: false
```

## Parameters

### dataUriToConvert

The `dataUriToString()` function expects a single string containing a valid data URI. The data URI
must start with `data:` and contain a comma separating the metadata from the encoded data. The
metadata must include `;base64`, and the data portion is decoded as base64. Data URIs without
`;base64` metadata are not supported and result in an error.

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
[06]: ./json.md
