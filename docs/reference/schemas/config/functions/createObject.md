---
description: Reference for the 'createObject' DSC configuration document function
ms.date:     07/28/2025
ms.topic:    reference
title:       createObject
---

## Synopsis

Creates a JSON object from key-value pairs.

## Syntax

```Syntax
createObject(<key1>, <value1>, <key2>, <value2>, ...)
```

## Description

The `createObject()` function creates a JSON object from the provided key-value pairs.
Arguments must be provided in pairs where the first argument of each pair is a string key,
and the second argument is the value of any type.

If no arguments are provided, the function returns an empty object. The number of arguments
must be even, as they represent key-value pairs.

## Examples

### Example 1 - Basic object creation

The following example shows how to create simple objects with string and numeric values.

```yaml
# createObject.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Basic object creation
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      simpleObject:     "[createObject('name', 'test')]"
      multipleProps:    "[createObject('key1', 'value1', 'key2', 42)]"
      emptyObject:      "[createObject()]"
```

```bash
dsc config get --file createObject.example.1.dsc.config.yaml
```

```yaml
results:
- name: Basic object creation
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        simpleObject:
          name: test
        multipleProps:
          key1: value1
          key2: 42
        emptyObject: {}
messages: []
hadErrors: false
```

### Example 2 - Mixed data types

The following example shows how to create objects with different value types.

```yaml
# createObject.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Mixed data types
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[createObject('string', 'hello', 'number', 123, 'boolean', true, 'nullValue', null())]"
```

```bash
dsc config get --file createObject.example.2.dsc.config.yaml
```

```yaml
results:
- name: Mixed data types
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        string: hello
        number: 123
        boolean: true
        nullValue: null
messages: []
hadErrors: false
```

### Example 3 - Nested objects and arrays

The following example shows how to create objects containing other objects and arrays.

```yaml
# createObject.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Nested structures
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      nestedObject:   "[createObject('config', createObject('timeout', 30, 'enabled', true))]"
      objectWithArray: "[createObject('items', createArray('foo', 'bar', 'baz'))]"
```

```bash
dsc config get --file createObject.example.3.dsc.config.yaml
```

```yaml
results:
- name: Nested structures
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        nestedObject:
          config:
            timeout: 30
            enabled: true
        objectWithArray:
          items:
          - foo
          - bar
          - baz
messages: []
hadErrors: false
```

### Example 4 - Using with other functions

The following example shows how to use `createObject()` with other DSC functions.

```yaml
# createObject.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  userName:
    type: string
    defaultValue: guest
resources:
- name: Function integration
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      userConfig:     "[createObject('user', parameters('userName'), 'role', coalesce(null(), 'default'))]"
      fallbackObject: "[createObject('result', coalesce(null(), createObject('status', 'success')))]"
```

```bash
dsc config get --file createObject.example.4.dsc.config.yaml
```

```yaml
results:
- name: Function integration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        userConfig:
          user: guest
          role: default
        fallbackObject:
          result:
            status: success
messages: []
hadErrors: false
```

## Parameters

### Key-value pairs

The `createObject()` function accepts zero or more key-value pairs. Each key must be a string,
and values can be of any type.

```yaml
Type:         key: [string], value: [any]
Required:     false
MinimumCount: 0
MaximumCount: unlimited (must be even number)
```

#### key

The object property name. Must be a string value.

```yaml
Type:     [string]
Required: true (when providing values)
```

#### value

The object property value. Can be any valid JSON type including strings, numbers, booleans, null, arrays, or other objects.

```yaml
Type:     [any]
Required: true (when providing keys)
```

## Output

The `createObject()` function returns a JSON object containing the specified key-value pairs.

```yaml
Type: [object]
```

## Error conditions

The function will return an error in the following cases:

- **Odd number of arguments**: Arguments must be provided in key-value pairs
- **Non-string keys**: All keys must be string values
- **Invalid argument types**: Arguments must be valid JSON types

## Notes

- Keys must be strings. If you specify numeric or other types, DSC raises an error.
- Values can be any valid JSON type including null, arrays, and nested objects.
- Duplicate keys will result in the last value overwriting previous values.
- Empty object creation (`createObject()` with no arguments) is supported.
- The function preserves the order of properties as specified.

## Related functions

- [`createArray()`][00] - Creates arrays that can be used as object values
- [`coalesce()`][01] - Provides fallback values for object properties
- [`null()`][02] - Creates explicit null values for object properties

<!-- Link reference definitions -->
[00]: ./createArray.md
[01]: ./coalesce.md
[02]: ./null.md
