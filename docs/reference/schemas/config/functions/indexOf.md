---
description: Reference for the 'indexOf' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       indexOf
---

# indexOf

## Synopsis

Returns the zero-based index of the first occurrence of an item in an array, or -1 if not found.

## Syntax

```Syntax
indexOf(<array>, <itemToFind>)
```

## Description

The `indexOf()` function searches an array for a specific item and returns the
zero-based index of the first matching element. If the item is not found, the
function returns `-1`. This is useful for determining the position of elements
in arrays or checking if an item exists without throwing errors.

The function performs strict equality checking:

- **Strings**: Case-sensitive exact match
- **Numbers**: Numeric equality comparison  
- **Arrays**: Deep equality (same length, order, and element values)
- **Objects**: Deep equality (same keys, values, and structure)

The search always starts from the beginning of the array and returns the index
of the first match found.

## Examples

### Example 1 - Locate a specific server in inventory

This example demonstrates finding the position of a database server within a
server inventory list, which could be useful for ordering deployment operations
or determining server priorities.

```yaml
# indexOf.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  servers:
    type: array
    defaultValue:
    - web01
    - web02
    - db01
resources:
- name: Find Server
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      dbIndex: "[indexOf(parameters('servers'), 'db01')]"
```

```bash
dsc config get --file indexOf.example.1.dsc.config.yaml
```

```yaml
results:
- name: Find Server
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        dbIndex: 2
messages: []
hadErrors: false
```

### Example 2 - Detect presence of a feature flag object

This example shows how to search for complex objects within an array, useful
for checking if specific configuration objects or feature flags are present
in a collection.

```yaml
# indexOf.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  flags:
    type: array
    defaultValue: []
resources:
- name: Flag Lookup
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      hasFeature: "[indexOf(array(createObject('name','Beta')), createObject('name','Beta'))]"
```

```bash
dsc config get --file indexOf.example.2.dsc.config.yaml
```

```yaml
results:
- name: Flag Lookup
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hasFeature: 0
messages: []
hadErrors: false
```

### Example 3 - Case sensitivity demonstration

This example illustrates the case-sensitive nature of string comparisons,
showing how exact case matching is required for successful string searches.

```yaml
# indexOf.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Case Demo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      appleLower: "[indexOf(createArray('Apple','Banana'), 'apple')]"
      appleExact: "[indexOf(createArray('Apple','Banana'), 'Apple')]"
```

```bash
dsc config get --file indexOf.example.3.dsc.config.yaml
```

```yaml
results:
- name: Case Demo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        appleLower: -1
        appleExact: 0
messages: []
hadErrors: false
```

## Parameters

### arrayToSearch

The array to search within.

```yaml
Type:     array
Required: true
```

### itemToFind

The item to search for within the array.

```yaml
Type:     string, number, array, or object
Required: true
```

The item must be one of the four supported data types. The function will
perform type-appropriate equality checking to find matching elements.

## Output

Returns the zero-based index of the first matching element, or -1 if not found.

```yaml
Type: number
```

## Related functions

- [`array()`][00] - Builds a heterogeneous array
- [`createArray()`][01] - Builds a homogeneous array
- [`first()`][02] - Gets the first element of an array
- [`contains()`][03] - Checks if an array, object key, or string contains a value

<!-- Link reference definitions -->
[00]: ./array.md
[01]: ./createArray.md
[02]: ./first.md
[03]: ./contains.md
