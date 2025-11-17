---
description: Reference for the 'array' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       array
---

# array

## Synopsis

Wraps a single value (string, number, array, or object) in an array.

## Syntax

```Syntax
array(<value>)
```

## Description

The `array()` function returns a new array containing the single input value.
The value can be a string, number, array, or object. Use
`createArray()` to construct arrays with multiple elements of the same type.

This function is useful when a schema expects an array, but you only have a
single value, or when you need to nest an existing array or object inside an
outer array.

## Examples

### Example 1 - Wrap an existing array as a single element

This example demonstrates wrapping an existing server list into a single array
element, producing a nested array. This can be useful when a downstream schema
expects an array of arrays.

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
    output: "[array(parameters('webServers'))]"
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
messages: []
hadErrors: false
```

### Example 2 - Wrap a single object for payloads

This example shows how to wrap a structured metadata object into an array for
logging or monitoring systems that expect arrays.

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
      payload: "[array(createObject('severity', 'info'))]"
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
        - severity: info
messages: []
hadErrors: false
```

### Example 3 - Wrap generated collections

This example demonstrates wrapping a generated collection (like one from
`createArray()` or `createObject()`) into an outer array.

```yaml
# array.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Combined Collections
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      combinedArray: "[array(createArray('a','b'))]"
      combinedObject: "[array(createObject('k','v'))]"
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
        combinedArray:
        - - a
          - b
        combinedObject:
        - k: v
messages: []
hadErrors: false
```

## Parameters

### value

The single value to wrap in the array.

```yaml
Type:         string, number, array, or object
Required:     false
MinimumCount: 1
MaximumCount: 1
```

The provided value will be wrapped into the resulting array. The value must be
one of the four accepted types: string, number (integer), array, or object.
Boolean and null values are not supported.

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
