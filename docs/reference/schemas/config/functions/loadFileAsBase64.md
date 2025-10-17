---
description: Reference for the 'loadFileAsBase64' DSC configuration document function
ms.date:     10/17/2025
ms.topic:    reference
title:       loadFileAsBase64
---

## Synopsis

Loads a file and encodes its content as a base64 string.

## Syntax

```Syntax
loadFileAsBase64(filePath)
```

## Description

The `loadFileAsBase64()` function reads a file and returns its content encoded
as a base64 string. File loading occurs during compilation, not at runtime.

- The file path can be absolute or relative to the configuration document.
- The maximum allowed file size is 96 KB (98,304 bytes).
- Both text and binary files are supported.
- The encoding is suitable for embedding binary data in text-based formats.

> [!IMPORTANT]
> This function loads file content at compile time. The file must be accessible
> when the configuration is compiled and executed. For runtime file operations, use an
> appropriate resource.

## Examples

### Example 1 - Embed a binary certificate file

Load a certificate file and encode it as base64 for transmission or storage.
This is useful for embedding certificates in configuration that will be decoded
at deployment time.

```yaml
# loadFileAsBase64.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo certificate
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[loadFileAsBase64('./certs/server.crt')]"
```

```bash
dsc config get --file loadFileAsBase64.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo certificate
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSURYVEN...
messages: []
hadErrors: false
```

### Example 2 - Load an image for API deployment

Encode an image file to send via an API that accepts base64-encoded images.
This is common with container registries, cloud APIs, and configuration
management systems.

```yaml
# loadFileAsBase64.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo logo image
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[loadFileAsBase64('./assets/logo.png')]"
```

```bash
dsc config get --file loadFileAsBase64.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo logo image
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: iVBORw0KGgoAAAANSUhEUgAAAAUA...
messages: []
hadErrors: false
```

### Example 3 - Embed encryption key

Load a binary encryption key file for secure configuration. Base64 encoding
makes binary data safe for JSON/YAML while preserving all bytes exactly.

```yaml
# loadFileAsBase64.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo encryption key
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[loadFileAsBase64('./secrets/encryption.key')]"
```

```bash
dsc config get --file loadFileAsBase64.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo encryption key
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: a2V5X2RhdGFfaGVyZQ==
messages: []
hadErrors: false
```

### Example 4 - Round-trip encoding and decoding

Load a text file as base64 and decode it using [`base64ToString()`][00]. This
demonstrates how to verify the encoding or process file content through base64.

```yaml
# loadFileAsBase64.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo decoded content
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[base64ToString(loadFileAsBase64('./README.md'))]"
```

```bash
dsc config get --file loadFileAsBase64.example.4.dsc.config.yaml
```

```yaml
results:
- name: Echo decoded content
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: |
        # My Application
        This is a sample README file.
messages: []
hadErrors: false
```

### Example 5 - Create data URI for embedding

Use [`concat()`][01] to create a data URI from an image file. This is useful
for embedding small images directly in HTML, CSS, or configuration files.

```yaml
# loadFileAsBase64.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo data URI
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat('data:image/png;base64,',
                     loadFileAsBase64('./icon.png'))]"
```

```bash
dsc config get --file loadFileAsBase64.example.5.dsc.config.yaml
```

```yaml
results:
- name: Echo data URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA...
messages: []
hadErrors: false
```

## Parameters

### filePath

The path to the file to load. Can be an absolute path or relative to the
configuration document location. Both text and binary files are supported.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

Returns the file content encoded as a base64 string. The maximum file size is
96 KB (98,304 bytes).

```yaml
Type: string
```

## Errors

The function returns an error when:

- The file path is not a string
- The file does not exist
- The path points to a directory instead of a file
- The file metadata cannot be read (permissions, etc.)
- The file size exceeds 96 KB (98,304 bytes)
- The file content cannot be read

## Related functions

- [`loadTextContent()`][02] - Loads text files with encoding support
- [`base64ToString()`][00] - Decodes base64 strings to text
- [`concat()`][01] - Concatenates strings together
- [`base64()`][03] - Encodes strings to base64

<!-- Link reference definitions -->
[00]: ./base64ToString.md
[01]: ./concat.md
[02]: ./loadTextContent.md
[03]: ./base64.md
