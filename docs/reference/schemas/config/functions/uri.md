---
description: Reference for the 'uri' DSC configuration document function
ms.date:     01/10/2025
ms.topic:    reference
title:       uri
---

# uri

## Synopsis

Creates an absolute URI by combining the baseUri and the relativeUri string.

## Syntax

```Syntax
uri(<baseUri>, <relativeUri>)
```

## Description

The `uri()` function combines a base URI with a relative URI to create an absolute URI according to
[RFC 3986](https://www.rfc-editor.org/rfc/rfc3986) URI resolution rules. This standardized behavior
ensures consistent and predictable URI construction.

### Requirements

- **Base URI must be absolute**: The base URI must include a scheme (such as `https://`, `http://`,
  or `file://`). Relative URIs or URIs without schemes return an error.
- **Base URI cannot be empty**: An empty base URI returns an error because an absolute URI requires
  a valid base.

### URI Resolution Behavior

The function follows RFC 3986 Section 5.2 (Relative Resolution) rules:

- **Absolute relative URIs**: If the relative URI contains a scheme (e.g.,
  `https://other.com/path`), it completely replaces the base URI.
- **Protocol-relative URIs** (starting with `//`): The relative URI inherits the scheme from the
  base URI. For example, `uri('https://example.com/', '//cdn.example.org/assets')` returns
  `https://cdn.example.org/assets`.
- **Path-absolute relative URIs** (starting with `/`): The relative path replaces the entire path
  of the base URI, keeping the scheme and authority. For example,
  `uri('https://example.com/old/path', '/new/path')` returns `https://example.com/new/path`.
- **Path-relative URIs** (not starting with `/`): The relative path is merged with the base URI's
  path. The last segment of the base path is removed and replaced with the relative URI. For
  example, `uri('https://example.com/api/v1', 'users')` returns `https://example.com/api/users`.
- **Empty relative URI**: Returns the base URI unchanged.

### Special Cases

- **Triple slash sequences** (`///`): Returns an error. Three or more consecutive slashes are
  invalid URI syntax.
- **Path normalization**: The function automatically normalizes paths, resolving `.` (current
  directory) and `..` (parent directory) references. For example,
  `uri('https://example.com/', 'path/../other')` returns `https://example.com/other`.
- **Query strings and fragments**: Query strings (`?query=value`) and fragments (`#section`) in the
  relative URI are preserved in the result.

Use this function to build API endpoints, file paths, or resource URLs dynamically from
configuration parameters.

## Examples

### Example 1 - Build API endpoint with trailing slash

The following example combines a base API URL ending with a slash with a relative path.

```yaml
# uri.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  apiBase:
    type: string
    defaultValue: https://api.example.com/v1/
  resourcePath:
    type: string
    defaultValue: users/123
resources:
- name: Build API endpoint
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      endpoint: "[uri(parameters('apiBase'), parameters('resourcePath'))]"
```

```bash
dsc config get --file uri.example.1.dsc.config.yaml
```

```yaml
results:
- name: Build API endpoint
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        endpoint: https://api.example.com/v1/users/123
messages: []
hadErrors: false
```

### Example 2 - Handle duplicate slashes

The following example shows how the function automatically handles cases where both the base URI
ends with a slash and the relative URI begins with a slash.

```yaml
# uri.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Combine URIs with duplicate slashes
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      withDuplicateSlashes: "[uri('https://example.com/', '/api/data')]"
      result: The function combines the slashes into one
```

```bash
dsc config get --file uri.example.2.dsc.config.yaml
```

```yaml
results:
- name: Combine URIs with duplicate slashes
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        withDuplicateSlashes: https://example.com/api/data
        result: The function combines the slashes into one
messages: []
hadErrors: false
```

### Example 3 - Replace path segments

The following example demonstrates how `uri()` replaces the last path segment when the base URI
doesn't end with a trailing slash. It uses the [`concat()`][01] function to build the base URL.

```yaml
# uri.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  currentVersion:
    type: string
    defaultValue: v1
  newVersion:
    type: string
    defaultValue: v2
resources:
- name: Update API version
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      oldEndpoint: "[concat('https://api.example.com/', parameters('currentVersion'))]"
      newEndpoint: "[uri(concat('https://api.example.com/', parameters('currentVersion')), parameters('newVersion'))]"
```

```bash
dsc config get --file uri.example.3.dsc.config.yaml
```

```yaml
results:
- name: Update API version
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        oldEndpoint: https://api.example.com/v1
        newEndpoint: https://api.example.com/v2
messages: []
hadErrors: false
```

### Example 4 - Build resource URLs

The following example shows how to use `uri()` with [`concat()`][01] to build complete resource
URLs from configuration parameters.

```yaml
# uri.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  storageAccount:
    type: string
    defaultValue: mystorageaccount
  containerName:
    type: string
    defaultValue: documents
  blobName:
    type: string
    defaultValue: report.pdf
resources:
- name: Build blob URL
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      blobUrl: >-
        [uri(
          concat('https://', parameters('storageAccount'), '.blob.core.windows.net/'),
          concat(parameters('containerName'), '/', parameters('blobName'))
        )]
```

```bash
dsc config get --file uri.example.4.dsc.config.yaml
```

```yaml
results:
- name: Build blob URL
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        blobUrl: https://mystorageaccount.blob.core.windows.net/documents/report.pdf
messages: []
hadErrors: false
```

### Example 5 - Handle query strings and ports

The following example demonstrates that `uri()` preserves query strings and port numbers correctly.

```yaml
# uri.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: URI with special components
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      withPort: "[uri('https://example.com:8080/', 'api')]"
      withQuery: "[uri('https://example.com/api/', 'search?q=test&limit=10')]"
```

```bash
dsc config get --file uri.example.5.dsc.config.yaml
```

```yaml
results:
- name: URI with special components
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        withPort: https://example.com:8080/api
        withQuery: https://example.com/api/search?q=test&limit=10
messages: []
hadErrors: false
```

### Example 6 - Protocol-relative URI

The following example shows how protocol-relative URIs (starting with `//`) inherit the scheme
from the base URI.

```yaml
# uri.example.6.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Protocol-relative URI
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      result: "[uri('https://example.com/', '//cdn.example.org/assets')]"
      explanation: The relative URI inherits the https scheme from the base
```

```bash
dsc config get --file uri.example.6.dsc.config.yaml
```

```yaml
results:
- name: Protocol-relative URI
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        result: https://cdn.example.org/assets
        explanation: The relative URI inherits the https scheme from the base
messages: []
hadErrors: false
```

## Parameters

### baseUri

The base URI string. Must be an absolute URI containing a scheme (such as `https://`, `http://`, or
`file://`). The function uses this as the foundation for resolving the relative URI according to
RFC 3986 rules.

```yaml
Type:     string
Required: true
Position: 1
```

### relativeUri

The relative URI string to combine with the base URI. Can be:

- An absolute URI (replaces the base entirely)
- A protocol-relative URI starting with `//` (inherits scheme from base)
- A path-absolute URI starting with `/` (replaces base path)
- A path-relative URI (merges with base path)
- An empty string (returns base unchanged)

This is combined with the base URI according to RFC 3986 URI resolution rules.

```yaml
Type:     string
Required: true
Position: 2
```

## Output

The `uri()` function returns a string containing the absolute URI created by combining the base URI
and relative URI according to RFC 3986 URI resolution rules.

```yaml
Type: string
```

## Related functions

- [`concat()`][01] - Concatenates multiple strings together
- [`format()`][02] - Creates a formatted string from a template
- [`substring()`][03] - Extracts a portion of a string
- [`replace()`][04] - Replaces text in a string
- [`split()`][05] - Splits a string into an array
- [`parameters()`][06] - Retrieves parameter values

<!-- Link reference definitions -->
[01]: ./concat.md
[02]: ./format.md
[03]: ./substring.md
[04]: ./replace.md
[05]: ./split.md
[06]: ./parameters.md
