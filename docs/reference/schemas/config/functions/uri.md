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

The `uri()` function combines a base URI with a relative URI to create an absolute URI. The
function handles trailing and leading slashes intelligently based on the structure of the base URI.

The behavior depends on whether the base URI ends with a trailing slash:

- **If baseUri ends with a trailing slash (`/`)**: The result is `baseUri` followed by
  `relativeUri`. If `relativeUri` also begins with a leading slash, the trailing and leading
  slashes are combined into one.

- **If baseUri doesn't end with a trailing slash**:
  - If `baseUri` has no slashes after the scheme (aside from `://` or `//` at the front), the
    result is `baseUri` followed by `relativeUri`.
  - If `baseUri` has slashes after the scheme but doesn't end with a slash, everything from the
    last slash onward is removed from `baseUri`, and the result is the modified `baseUri` followed
    by `relativeUri`.

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

## Parameters

### baseUri

The base URI string. The function's behavior depends on whether this value ends with a trailing
slash.

```yaml
Type:     string
Required: true
Position: 1
```

### relativeUri

The relative URI string to add to the base URI string. This is combined with the base URI according
to the rules described in the Description section.

```yaml
Type:     string
Required: true
Position: 2
```

## Output

The `uri()` function returns a string containing the absolute URI created by combining the base URI
and relative URI according to the slash-handling rules.

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
