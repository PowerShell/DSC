---
description: Reference for the 'tryGet' DSC configuration document function
ms.date:     01/29/2025
ms.topic:    reference
title:       tryGet
---

## Synopsis

Safely retrieves a value from an array by index or an object by key without
throwing an error if the key or index doesn't exist.

## Syntax

```Syntax
tryGet(source, keyOrIndex)
```

## Description

The `tryGet()` function provides a safe way to access elements in arrays or
objects without causing errors when the key or index doesn't exist. Unlike
direct property access that might fail, this function returns `null` when the
requested element cannot be found.

For arrays, the function uses zero-based indexing where `0` refers to the first
element. For objects, it retrieves values by their property key name. This is
particularly useful when working with dynamic data structures where the presence
of keys or the length of arrays isn't guaranteed.

The function returns `null` in the following cases:

- The specified key doesn't exist in the object
- The array index is negative
- The array index is greater than or equal to the array length
- The array is empty

## Examples

### Example 1 - Safely access configuration settings with fallbacks

Access optional configuration values that might not be present in all
environments without causing errors. This example demonstrates retrieving
feature flags that may or may not be defined. This example uses
[`createObject()`][03] to build the configuration object and [`coalesce()`][02]
for fallback values.

```yaml
# tryGet.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Feature Flags
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      config: "[createObject('enableBeta', true, 'enableDebug', false)]"
      betaEnabled: "[coalesce(tryGet(createObject('enableBeta', true, 'enableDebug', false), 'enableBeta'), false)]"
      alphaEnabled: "[coalesce(tryGet(createObject('enableBeta', true, 'enableDebug', false), 'enableAlpha'), false)]"
      debugEnabled: "[tryGet(createObject('enableBeta', true, 'enableDebug', false), 'enableDebug')]"
```

```bash
dsc config get --file tryGet.example.1.dsc.config.yaml
```

```yaml
results:
- name: Feature Flags
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        config:
          enableBeta: true
          enableDebug: false
        betaEnabled: true
        alphaEnabled: false
        debugEnabled: false
messages: []
hadErrors: false
```

The function safely returns `true` for the existing `enableBeta` flag, `null`
for the non-existent `enableAlpha` flag (which `coalesce()` converts to
`false`), and `false` for the `enableDebug` flag.

### Example 2 - Access deployment environment settings

Retrieve environment-specific configuration values with safe defaults. This is
useful when different environments might have different configuration keys. This
example uses [`createObject()`][03] to build the environment settings and
demonstrates accessing nested object properties.

```yaml
# tryGet.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Environment Config
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      environments: "[createObject('production', createObject('replicas', 5, 'region', 'us-east-1'), 'staging', createObject('replicas', 2))]"
      productionEnv: "[tryGet(createObject('production', createObject('replicas', 5, 'region', 'us-east-1'), 'staging', createObject('replicas', 2)), 'production')]"
      stagingEnv: "[tryGet(createObject('production', createObject('replicas', 5, 'region', 'us-east-1'), 'staging', createObject('replicas', 2)), 'staging')]"
      developmentEnv: "[tryGet(createObject('production', createObject('replicas', 5, 'region', 'us-east-1'), 'staging', createObject('replicas', 2)), 'development')]"
      prodReplicas: "[tryGet(createObject('replicas', 5, 'region', 'us-east-1'), 'replicas')]"
      prodRegion: "[tryGet(createObject('replicas', 5, 'region', 'us-east-1'), 'region')]"
      stagingRegion: "[tryGet(createObject('replicas', 2), 'region')]"
```

```bash
dsc config get --file tryGet.example.2.dsc.config.yaml
```

```yaml
results:
- name: Environment Config
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        environments:
          production:
            replicas: 5
            region: us-east-1
          staging:
            replicas: 2
        productionEnv:
          replicas: 5
          region: us-east-1
        stagingEnv:
          replicas: 2
        developmentEnv: null
        prodReplicas: 5
        prodRegion: us-east-1
        stagingRegion: null
messages: []
hadErrors: false
```

The function safely returns the environment objects when they exist, or `null`
for the non-existent `development` environment. It also demonstrates accessing
nested object properties separately.

### Example 3 - Safely access array elements by position

Access array elements without worrying about index out-of-range errors. This is
particularly useful when processing arrays of unknown or varying length. This
example uses [`createArray()`][00] to build the version history.

```yaml
# tryGet.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Version History
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      versions: "[createArray('v1.0.0', 'v1.1.0', 'v1.2.0')]"
      current: "[tryGet(createArray('v1.0.0', 'v1.1.0', 'v1.2.0'), 2)]"
      previous: "[tryGet(createArray('v1.0.0', 'v1.1.0', 'v1.2.0'), 1)]"
      future: "[tryGet(createArray('v1.0.0', 'v1.1.0', 'v1.2.0'), 5)]"
      invalid: "[tryGet(createArray('v1.0.0', 'v1.1.0', 'v1.2.0'), -1)]"
```

```bash
dsc config get --file tryGet.example.3.dsc.config.yaml
```

```yaml
results:
- name: Version History
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        versions:
        - v1.0.0
        - v1.1.0
        - v1.2.0
        current: v1.2.0
        previous: v1.1.0
        future: null
        invalid: null
messages: []
hadErrors: false
```

The function safely returns array elements at valid indices, and `null` for
out-of-range indices (both negative and beyond the array length).

### Example 4 - Parse API responses with optional fields

Process data structures that might have optional or conditional fields, such as
API responses where certain fields only appear in specific scenarios. This
example uses [`createObject()`][03] to simulate API response data.

```yaml
# tryGet.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: API Response Parser
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      successResponse: "[createObject('status', 200, 'data', createObject('id', 123, 'name', 'example'))]"
      errorResponse: "[createObject('status', 404, 'error', 'Not Found')]"
      successData: "[tryGet(createObject('status', 200, 'data', createObject('id', 123, 'name', 'example')), 'data')]"
      successError: "[tryGet(createObject('status', 200, 'data', createObject('id', 123, 'name', 'example')), 'error')]"
      errorData: "[tryGet(createObject('status', 404, 'error', 'Not Found'), 'data')]"
      errorMessage: "[tryGet(createObject('status', 404, 'error', 'Not Found'), 'error')]"
```

```bash
dsc config get --file tryGet.example.4.dsc.config.yaml
```

```yaml
results:
- name: API Response Parser
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        successResponse:
          status: 200
          data:
            id: 123
            name: example
        errorResponse:
          status: 404
          error: Not Found
        successData:
          id: 123
          name: example
        successError: null
        errorData: null
        errorMessage: Not Found
messages: []
hadErrors: false
```

The function gracefully handles missing fields, allowing you to build robust
parsers that work with varying response structures.

## Parameters

### source

The array or object to retrieve the value from. For arrays, elements are
accessed by zero-based index. For objects, values are accessed by string key.
Required.

```yaml
Type:     array | object
Required: true
Position: 1
```

### keyOrIndex

For arrays: the zero-based integer index of the element to retrieve. For
objects: the string key name of the property to retrieve. Required.

```yaml
Type:     integer | string
Required: true
Position: 2
```

## Output

Returns the value at the specified index or key if it exists. Returns `null` if:

- The array index is negative, out of range, or the array is empty
- The object key doesn't exist

The return type matches the type of the element or property value in the
array or object.

```yaml
Type: any | null
```

## Errors

The function returns an error in the following cases:

- **Invalid source type**: The first argument is neither an array nor an object
- **Invalid key type for object**: When accessing an object, the second argument
  must be a string
- **Invalid index type for array**: When accessing an array, the second argument
  must be an integer

## Related functions

- [`createArray()`][00] - Creates an array from provided values
- [`createObject()`][03] - Creates an object from key-value pairs
- [`coalesce()`][02] - Returns the first non-null value from a list
- [`if()`][04] - Returns one of two values based on a condition
- [`equals()`][05] - Compares two values for equality
- [`not()`][06] - Inverts a boolean value
- [`tryIndexFromEnd()`][01] - Safely retrieves array elements by counting backward from the end

<!-- Link reference definitions -->
[00]: ./createArray.md
[01]: ./tryIndexFromEnd.md
[02]: ./coalesce.md
[03]: ./createObject.md
[04]: ./if.md
[05]: ./equals.md
[06]: ./not.md
