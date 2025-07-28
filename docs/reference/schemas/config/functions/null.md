---
description: Reference for the 'null' DSC configuration document function
ms.date:     07/28/2025
ms.topic:    reference
title:       null
---

# null

## Synopsis

Returns a null value.

## Syntax

```Syntax
null()
```

## Description

The `null()` function returns a JSON null value. This function takes no arguments
and always returns null. It's useful for explicitly setting null values in configurations,
testing null handling in other functions, or providing null fallbacks in conditional
expressions.

## Examples

### Example 1 - Basic null usage

The following example shows basic usage of the null function.

```yaml
# null.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Null value
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[null()]"
```

```bash
dsc config get --file null.example.1.dsc.config.yaml
```

```yaml
results:
- name: Null value
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: null
messages: []
hadErrors: false
```

### Example 2 - Null in object creation

The following example shows how to use null when creating objects with null properties.

```yaml
# null.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Object with null property
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[createObject('name', 'test', 'value', null(), 'active', true)]"
```

```bash
dsc config get --file null.example.2.dsc.config.yaml
```

```yaml
results:
- name: Object with null property
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        name: test
        value: null
        active: true
messages: []
hadErrors: false
```

### Example 3 - Null with coalesce function

The following example shows how null works with the coalesce function for fallback scenarios.

```yaml
# null.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Coalesce with null
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      fallbackValue: "[coalesce(null(), 'default-value')]"
      nestedObject:  "[createObject('result', coalesce(null(), 'fallback'))]"
      multipleNulls: "[coalesce(null(), null(), null(), 'final-fallback')]"
```

```bash
dsc config get --file null.example.3.dsc.config.yaml
```

```yaml
results:
- name: Coalesce with null
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        fallbackValue: default-value
        nestedObject:
          result: fallback
        multipleNulls: final-fallback
messages: []
hadErrors: false
```

## Parameters

The `null()` function accepts no parameters.

```yaml
Type:         none
Required:     false
MinimumCount: 0
MaximumCount: 0
```

## Output

The `null()` function always returns a JSON null value.

```yaml
Type: null
```

## Notes

- The `null()` function is particularly useful when working with other functions that handle null values
  , such as `coalesce()`
- Unlike undefined or missing values, `null()` explicitly represents the JSON null value
- When used in object creation with `createObject()`, null properties are included in the resulting object
- The function takes no arguments and will return an error if any arguments are provided

## Related functions

- [`coalesce()`][00] - Returns the first non-null value from a list of arguments
- [`createObject()`][01] - Creates objects that can contain null properties

<!-- Link reference definitions -->
[00]: ./coalesce.md
[01]: ./createObject.md
