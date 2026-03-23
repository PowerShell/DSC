---
description: Reference for the 'objectKeys' DSC configuration document function
ms.date:     11/14/2025
ms.topic:    reference
title:       objectKeys
---

## Synopsis

Returns an array containing all the keys from an object.

## Syntax

```Syntax
objectKeys(<inputObject>)
```

## Description

The `objectKeys()` function extracts all property names from an object and returns them as
an array of strings. This function is useful for:

- Iterating over object properties when you only need the keys
- Counting the number of properties in an object
- Checking if specific keys exist in an object
- Converting object keys for further processing

The function only returns the top-level keys of the object. For nested objects, only the
outer keys are included in the result.

This function is similar to [`items()`][00], which returns both keys and values, while
`objectKeys()` returns only the keys.

## Examples

### Example 1 - Extract keys from simple object

The following example extracts all keys from a simple object.

```yaml
# objectKeys.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[objectKeys(createObject('firstName', 'John', 'lastName', 'Doe', 'age', 30))]"
```

```bash
dsc config get --file objectKeys.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - firstName
      - lastName
      - age
messages: []
hadErrors: false
```

### Example 2 - Count object properties

The following example uses `objectKeys()` with [`length()`][01] to count the number of
properties in an object.

```yaml
# objectKeys.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[length(objectKeys(createObject('a', 1, 'b', 2, 'c', 3)))]"
```

```bash
dsc config get --file objectKeys.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 3
messages: []
hadErrors: false
```

### Example 3 - Check if key exists

The following example uses `objectKeys()` with [`contains()`][02] to check if a specific
key exists in an object.

```yaml
# objectKeys.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  config:
    type: object
    defaultValue:
      enabled: true
      timeout: 30
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      hasEnabled: "[contains(objectKeys(parameters('config')), 'enabled')]"
      hasDebug: "[contains(objectKeys(parameters('config')), 'debug')]"
```

```bash
dsc config get --file objectKeys.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hasEnabled: true
        hasDebug: false
messages: []
hadErrors: false
```

### Example 4 - Iterate over keys with copy loop

The following example uses `objectKeys()` to iterate over object properties using the
[`copy`][03] feature.

```yaml
# objectKeys.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  settings:
    type: object
    defaultValue:
      debug: false
      logLevel: info
      maxRetries: 3
resources:
- name: "[format('Setting-{0}', copyIndex())]"
  copy:
    name: settingsLoop
    count: "[length(objectKeys(parameters('settings')))]"
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[objectKeys(parameters('settings'))[copyIndex()]]"
```

```bash
dsc config get --file objectKeys.example.4.dsc.config.yaml
```

```yaml
results:
- name: Setting-0
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: debug
- name: Setting-1
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: logLevel
- name: Setting-2
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: maxRetries
messages: []
hadErrors: false
```

### Example 5 - Top-level keys only

The following example demonstrates that `objectKeys()` only returns top-level keys, even
when the object contains nested objects.

```yaml
# objectKeys.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[objectKeys(createObject('user', createObject('name', 'John', 'age', 30), 'role', 'admin'))]"
```

```bash
dsc config get --file objectKeys.example.5.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - user
      - role
messages: []
hadErrors: false
```

### Example 6 - Empty object

The following example shows that `objectKeys()` returns an empty array for an empty object.

```yaml
# objectKeys.example.6.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      keys: "[objectKeys(createObject())]"
      isEmpty: "[equals(length(objectKeys(createObject())), 0)]"
```

```bash
dsc config get --file objectKeys.example.6.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        keys: []
        isEmpty: true
messages: []
hadErrors: false
```

## Parameters

### inputObject

The object from which to extract the keys.

```yaml
Type:     object
Required: true
Position: 1
```

## Output

Returns an array of strings, where each string is a property name (key) from the input
object. The array contains only the top-level keys.

```yaml
Type: array
```

## Error conditions

The function will return an error in the following cases:

- **Not an object**: The input is not an object (e.g., string, number, array, null)

## Notes

- The function only returns top-level keys; nested object keys are not included
- For empty objects, the function returns an empty array
- The order of keys in the returned array follows JSON object property ordering
- Key names are always returned as strings
- To get both keys and values, use [`items()`][00] instead

## Related functions

- [`items()`][00] - Converts an object to an array of key-value pairs
- [`createObject()`][04] - Creates an object from key-value pairs
- [`length()`][01] - Returns the number of elements in an array
- [`contains()`][02] - Checks if an array contains a specific value

<!-- Link reference definitions -->
[00]: ./items.md
[01]: ./length.md
[02]: ./contains.md
[03]: ../document/copy.md
[04]: ./createObject.md
