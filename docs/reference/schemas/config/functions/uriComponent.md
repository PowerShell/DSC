---
description: Reference for the 'uriComponent' DSC configuration document function
ms.date:     01/10/2025
ms.topic:    reference
title:       uriComponent
---

# uriComponent

## Synopsis

Encodes a string for use as a URI component using percent-encoding.

## Syntax

```Syntax
uriComponent(<stringToEncode>)
```

## Description

The `uriComponent()` function encodes a string using percent-encoding (also known as URL encoding)
to make it safe for use as a component of a URI. The function encodes all characters except the
unreserved characters defined in RFC 3986:

- **Unreserved characters** (not encoded): `A-Z`, `a-z`, `0-9`, `-`, `_`, `.`, `~`
- **All other characters** are percent-encoded as `%XX` where `XX` is the hexadecimal value

Use this function when you need to include user-provided data, special characters, or spaces in
URLs, query strings, or other URI components. This ensures that the resulting URI is valid and that
special characters don't break the URI structure.

## Examples

### Example 1 - Encode query parameter value

The following example shows how to encode a string containing spaces for use in a URL query
parameter.

```yaml
# uricomponent.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  searchTerm:
    type: string
    defaultValue: hello world
resources:
- name: Build search URL
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      original: "[parameters('searchTerm')]"
      encoded: "[uriComponent(parameters('searchTerm'))]"
      fullUrl: "[concat('https://example.com/search?q=', uriComponent(parameters('searchTerm')))]"
```

```bash
dsc config get --file uricomponent.example.1.dsc.config.yaml
```

```yaml
results:
- name: Build search URL
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        original: hello world
        encoded: hello%20world
        fullUrl: https://example.com/search?q=hello%20world
messages: []
hadErrors: false
```

### Example 2 - Encode email address

The following example demonstrates encoding an email address that contains special characters.

```yaml
# uricomponent.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  email:
    type: string
    defaultValue: user+tag@example.com
resources:
- name: Encode email for URL
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      encoded: "[uriComponent(parameters('email'))]"
      mailtoLink: "[concat('mailto:', uriComponent(parameters('email')))]"
```

```bash
dsc config get --file uricomponent.example.2.dsc.config.yaml
```

```yaml
results:
- name: Encode email for URL
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        encoded: user%2Btag%40example.com
        mailtoLink: mailto:user%2Btag%40example.com
messages: []
hadErrors: false
```

### Example 3 - Encode complete URL

The following example shows how `uriComponent()` encodes an entire URL, including the protocol,
slashes, and special characters.

```yaml
# uricomponent.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Encode complete URL
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      originalUrl: https://example.com/path?query=value
      encodedUrl: "[uriComponent('https://example.com/path?query=value')]"
```

```bash
dsc config get --file uricomponent.example.3.dsc.config.yaml
```

```yaml
results:
- name: Encode complete URL
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        originalUrl: https://example.com/path?query=value
        encodedUrl: https%3A%2F%2Fexample.com%2Fpath%3Fquery%3Dvalue
messages: []
hadErrors: false
```

### Example 4 - Build API request with encoded parameters

The following example demonstrates using `uriComponent()` with [`concat()`][01] and [`uri()`][02]
to build an API URL with safely encoded query parameters.

```yaml
# uricomponent.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  apiBase:
    type: string
    defaultValue: https://api.example.com
  resourcePath:
    type: string
    defaultValue: /users/search
  nameFilter:
    type: string
    defaultValue: John Doe
  ageFilter:
    type: string
    defaultValue: '30'
resources:
- name: Build API URL with query string
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      apiUrl: >-
        [concat(
          uri(parameters('apiBase'), parameters('resourcePath')),
          '?name=',
          uriComponent(parameters('nameFilter')),
          '&age=',
          parameters('ageFilter')
        )]
```

```bash
dsc config get --file uricomponent.example.4.dsc.config.yaml
```

```yaml
results:
- name: Build API URL with query string
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        apiUrl: https://api.example.com/users/search?name=John%20Doe&age=30
messages: []
hadErrors: false
```

### Example 5 - Unreserved characters remain unchanged

The following example shows that unreserved characters (letters, numbers, hyphen, underscore,
period, and tilde) are not encoded.

```yaml
# uricomponent.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Unreserved character handling
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      original: ABCabc123-_.~
      encoded: "[uriComponent('ABCabc123-_.~')]"
      identical: "[equals(uriComponent('ABCabc123-_.~'), 'ABCabc123-_.~')]"
```

```bash
dsc config get --file uricomponent.example.5.dsc.config.yaml
```

```yaml
results:
- name: Unreserved character handling
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        original: ABCabc123-_.~
        encoded: ABCabc123-_.~
        identical: true
messages: []
hadErrors: false
```

## Parameters

### stringToEncode

The string value to encode using percent-encoding. All characters except unreserved characters
(A-Z, a-z, 0-9, -, _, ., ~) are encoded.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

The `uriComponent()` function returns a string with all characters except unreserved characters
replaced with their percent-encoded equivalents (e.g., space becomes `%20`, `@` becomes `%40`).

```yaml
Type: string
```

## Related functions

- [`uri()`][02] - Combines base and relative URIs
- [`concat()`][01] - Concatenates multiple strings together
- [`format()`][03] - Creates a formatted string from a template
- [`base64()`][04] - Encodes a string to base64
- [`parameters()`][05] - Retrieves parameter values
- [`equals()`][06] - Compares two values for equality

<!-- Link reference definitions -->
[01]: ./concat.md
[02]: ./uri.md
[03]: ./format.md
[04]: ./base64.md
[05]: ./parameters.md
[06]: ./equals.md
