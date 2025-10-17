---
description: Reference for the 'loadTextContent' DSC configuration document function
ms.date:     10/17/2025
ms.topic:    reference
title:       loadTextContent
---

## Synopsis

Loads the content of a text file into a string with optional encoding support.

## Syntax

```Syntax
loadTextContent(filePath)
loadTextContent(filePath, encoding)
```

## Description

The `loadTextContent()` function reads the contents of a text file and returns
it as a string. Content loading occurs during compilation, not at runtime.

- The file path can be absolute or relative to the configuration document.
- The maximum allowed content size is 131,072 characters (including line
  endings).
- Line endings are preserved as they appear in the source file.
- If no encoding is specified, UTF-8 is used by default.

> [!IMPORTANT]
> This function loads file content at compile time. The file must be accessible
> when the configuration is compiled and executed. For runtime file operations, use an
> appropriate resource.

## Examples

### Example 1 - Load a configuration template

Load a configuration file template to use as input for another resource. This
is useful for managing complex configuration files that are easier to maintain
as separate files.

```yaml
# loadTextContent.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo nginx config
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[loadTextContent('./templates/nginx.conf')]"
```

```bash
dsc config get --file loadTextContent.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo nginx config
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: |
        server {
            listen 80;
            server_name example.com;
            root /var/www/html;
        }
messages: []
hadErrors: false
```

### Example 2 - Load SSH public key for deployment

Load an SSH public key from a file to configure user access. This keeps
sensitive key material in separate files rather than embedded in
configurations.

```yaml
# loadTextContent.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo public key
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[loadTextContent('./keys/id_rsa.pub')]"
```

```bash
dsc config get --file loadTextContent.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo public key
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAAB... user@host
messages: []
hadErrors: false
```

### Example 3 - Embed script content

Load a shell script or PowerShell script to execute via a resource. This allows
you to maintain scripts in separate files with proper syntax highlighting and
version control.

```yaml
# loadTextContent.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo setup script
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[loadTextContent('./scripts/setup.sh')]"
```

```bash
dsc config get --file loadTextContent.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo setup script
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: |
        #!/bin/bash
        set -e
        apt-get update
        apt-get install -y nginx
messages: []
hadErrors: false
```

### Example 4 - Load file with specific encoding

Load a legacy configuration file that uses ISO-8859-1 encoding. This is useful
when working with files created on older systems or in specific locales.

```yaml
# loadTextContent.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo legacy config
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[loadTextContent('./legacy/app.conf', 'iso-8859-1')]"
```

```bash
dsc config get --file loadTextContent.example.4.dsc.config.yaml
```

```yaml
results:
- name: Echo legacy config
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "application_name=Café Manager\nversion=1.0"
messages: []
hadErrors: false
```

### Example 5 - Combine with other functions

Use [`concat()`][01] to add a header or footer to loaded content. This example
loads a certificate and wraps it with additional metadata.

```yaml
# loadTextContent.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo cert with metadata
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat('# Certificate for example.com\n',
                     loadTextContent('./certs/example.crt'),
                     '\n# End of certificate')]"
```

```bash
dsc config get --file loadTextContent.example.5.dsc.config.yaml
```

```yaml
results:
- name: Echo cert with metadata
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: |
        # Certificate for example.com
        -----BEGIN CERTIFICATE-----
        MIIDXTCCAkWgAwIBAgIJAKJ...
        -----END CERTIFICATE-----
        # End of certificate
messages: []
hadErrors: false
```

## Parameters

### filePath

The path to the text file to load. Can be an absolute path or relative to the
configuration document location.

```yaml
Type:     string
Required: true
Position: 1
```

### encoding

The character encoding of the file. If not provided, UTF-8 is used.

Supported encodings:

- `utf-8` - UTF-8 encoding (default)
- `utf-16` - UTF-16 Little Endian encoding
- `utf-16BE` - UTF-16 Big Endian encoding
- `iso-8859-1` - ISO-8859-1 / Latin-1 encoding
- `us-ascii` - US-ASCII encoding (7-bit ASCII)

```yaml
Type:     string
Required: false
Position: 2
Default:  utf-8
```

## Output

Returns the file content as a string with a maximum of 131,072 characters.

```yaml
Type: string
```

## Errors

The function returns an error when:

- The file path is not a string
- The file does not exist
- The path points to a directory instead of a file
- The file cannot be read (permissions, etc.)
- The encoding parameter is not a string
- An unsupported encoding is specified
- The file content cannot be decoded with the specified encoding
- The file content exceeds 131,072 characters

## Related functions

- [`loadFileAsBase64()`][00] - Loads a file as base64-encoded string
- [`concat()`][01] - Concatenates strings together
- [`base64ToString()`][02] - Decodes base64 strings to text

<!-- Link reference definitions -->
[00]: ./loadFileAsBase64.md
[01]: ./concat.md
[02]: ./base64ToString.md
