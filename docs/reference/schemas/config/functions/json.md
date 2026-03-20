---
description: Reference for the 'json' DSC configuration document function
ms.date:     10/11/2025
ms.topic:    reference
title:       json
---

## Synopsis

Converts a valid JSON string into a JSON data type.

## Syntax

```Syntax
json(arg1)
```

## Description

The `json()` function parses a JSON string and returns the corresponding JSON data type.

- The string must be a properly formatted JSON string.
- Returns the parsed JSON value (object, array, string, number, boolean, or null).

This function is useful for converting JSON strings received from external sources or
stored as configuration into usable data structures.

## Examples

### Example 1 - Parse JSON object

This example parses a JSON string containing an object into a usable object data type.

```yaml
# json.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[json('{\"name\":\"John\",\"age\":30}')]"
```

```bash
dsc config get --file json.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        name: John
        age: 30
messages: []
hadErrors: false
```

### Example 2 - Parse JSON array

This example parses a JSON string containing an array using [`json()`][00] and then
uses [`length()`][01] to count the elements.

```yaml
# json.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[length(json('[1,2,3,4,5]'))]"
```

```bash
dsc config get --file json.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 5
messages: []
hadErrors: false
```

### Example 3 - Parse nested JSON structure

This example parses a JSON string with nested objects and arrays, then accesses nested
properties using array indexing and property access.

```yaml
# json.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[json('{\"users\":[{\"name\":\"Alice\"},{\"name\":\"Bob\"}]}').users[0].name]"
```

```bash
dsc config get --file json.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Alice
messages: []
hadErrors: false
```

### Example 4 - Parse JSON with whitespace

This example shows that `json()` handles JSON strings with extra whitespace.

```yaml
# json.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[json('  { \"key\" : \"value\" }  ').key]"
```

```bash
dsc config get --file json.example.4.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: value
messages: []
hadErrors: false
```

### Example 5 - Parse primitive JSON values

This example demonstrates parsing different primitive JSON values including strings,
numbers, and booleans.

```yaml
# json.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo string
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[json('\"hello\"')]"
- name: Echo number
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[json('42')]"
- name: Echo boolean
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[json('true')]"
```

```bash
dsc config get --file json.example.5.dsc.config.yaml
```

```yaml
results:
- name: Echo string
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: hello
- name: Echo number
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 42
- name: Echo boolean
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: true
messages: []
hadErrors: false
```

## Parameters

### arg1

The JSON string to parse. Must be a properly formatted JSON string.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

Returns the parsed JSON value. The type depends on the JSON content:

- Object for JSON objects
- Array for JSON arrays
- String for JSON strings
- Number for JSON numbers
- Boolean for JSON booleans
- Null for JSON null

```yaml
Type: object | array | string | number | boolean | null
```

## Related functions

- [`length()`][00] - Returns the length of an array or object
- [`string()`][01] - Converts values to strings

<!-- Link reference definitions -->
[00]: ./length.md
[01]: ./string.md
