---
description: Reference for the 'array' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       array
---

# array

## Synopsis

Creates an array from the given elements (strings, numbers, arrays, or objects).

## Syntax

```Syntax
array(<element1>, <element2>, ...)
```

## Description

The `array()` function creates an array from the provided arguments, allowing
mixed data types within the same array. Unlike `createArray()` which requires
all elements to be the same type, `array()` accepts any combination of strings,
numbers, arrays, and objects as arguments.

This function is useful when you need to combine different types of data into a
single collection, such as mixing configuration values, computed results, and
structured metadata in deployment scenarios.

## Examples

### Example 1 - Build a deployment plan

This example demonstrates combining different data types to create a comprehensive
deployment configuration. The array contains an existing server list, a numeric
batch size, and a configuration object with deployment strategy details.

```yaml
# array.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  webServers:
    type: array
    defaultValue:
    - web01
    - web02
  batchSize:
    type: int
    defaultValue: 2
resources:
- name: Deployment Plan
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[array(parameters('webServers'), parameters('batchSize'), createObject('strategy', 'blue-green'))]"
```

```bash
dsc config get --file array.example.1.dsc.config.yaml
```

```yaml
results:
- name: Deployment Plan
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - - web01
        - web02
      - 2
      - strategy: blue-green
messages: []
hadErrors: false
```

### Example 2 - Compose mixed telemetry payload parts

This example shows how to construct a telemetry payload by combining a string
identifier, structured metadata object, and numeric status code into a single
array for logging or monitoring systems.

```yaml
# array.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  correlationId:
    type: string
    defaultValue: ABC123
resources:
- name: Telemetry Payload
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      payload: "[array(parameters('correlationId'), createObject('severity', 'info'), 200)]"
```

```bash
dsc config get --file array.example.2.dsc.config.yaml
```

```yaml
results:
- name: Telemetry Payload
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        payload:
        - ABC123
        - severity: info
        - 200
messages: []
hadErrors: false
```

### Example 3 - Combine generated and literal collections

This example demonstrates nesting arrays and objects within a parent array,
showing how `array()` can combine results from other DSC functions like
`createArray()` and `createObject()` with literal values.

```yaml
# array.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Combined Collections
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      combined: "[array(createArray('a','b'), array(1,2), createObject('k','v'))]"
```

```bash
dsc config get --file array.example.3.dsc.config.yaml
```

```yaml
results:
- name: Combined Collections
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        combined:
        - - a
          - b
        - - 1
          - 2
        - k: v
messages: []
hadErrors: false
```

## Parameters

### element1, element2, ...

The elements to include in the array.

```yaml
Type:         string, number, array, or object
Required:     false
Multiplicity: 0 or more
```

Each element provided as an argument will be added to the resulting array in the
order specified. All elements must be one of the four accepted types: string,
number (integer), array, or object. Boolean and null values are not supported.

## Output

Returns a new array containing the provided elements in order.

```yaml
Type: array
```

## Related functions

- [`createArray()`][00] - Creates a homogeneous array (all elements same type)
- [`createObject()`][01] - Builds an object from key/value pairs
- [`first()`][02] - Gets the first element from an array
- [`indexOf()`][03] - Finds the index of an item in an array

<!-- Link reference definitions -->
[00]: ./createArray.md
[01]: ./createObject.md
[02]: ./first.md
[03]: ./indexOf.md
