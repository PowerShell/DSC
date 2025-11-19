---
description: Reference for the 'shallowMerge' DSC configuration document function
ms.date:     11/19/2025
ms.topic:    reference
title:       shallowMerge
---

## Synopsis

Combines an array of objects into a single object where only the top-level properties are merged.

## Syntax

```Syntax
shallowMerge(<inputArray>)
```

## Description

The `shallowMerge()` function takes an array of objects and combines them into a single
object by merging their properties. When the same property name appears in multiple objects,
the value from the last object in the array takes precedence.

This is a **shallow merge**, meaning:

- Top-level properties are merged from all objects
- If a property value is an object, it replaces the entire object from previous objects
  rather than merging the nested properties
- Arrays and other complex types are also replaced entirely, not combined

This function is useful for:

- Building composite configuration objects from multiple sources
- Applying configuration overrides where later values take precedence
- Combining default settings with user-specified customizations
- Merging environment-specific configurations

The shallow merge behavior differs from a deep merge (like [`union()`][00]) where nested
objects would be recursively merged. With `shallowMerge()`, nested structures are replaced
entirely by the last object's value.

## Examples

### Example 1 - Merge configuration objects

The following example demonstrates merging two configuration objects using [`createArray()`][02]
and [`createObject()`][03], where the second object overrides properties from the first.

```yaml
# shallowMerge.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[shallowMerge(createArray(createObject('host', 'localhost', 'port', 8080), createObject('port', 9000, 'ssl', true())))]"
```

```bash
dsc config get --file shallowMerge.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        host: localhost
        port: 9000
        ssl: true
messages: []
hadErrors: false
```

Notice how the `port` value from the second object (9000) replaces the value from the first
object (8080), while properties that only exist in one object (`host` and `ssl`) are
preserved.

### Example 2 - Apply multiple configuration layers

The following example shows combining multiple configuration layers using parameters, where
later objects in the array override properties from earlier objects.

```yaml
# shallowMerge.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  defaults:
    type: object
    defaultValue:
      timeout: 30
      retries: 3
      debug: false
  environment:
    type: object
    defaultValue:
      timeout: 60
  userPrefs:
    type: object
    defaultValue:
      debug: true
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[shallowMerge(createArray(parameters('defaults'), parameters('environment'), parameters('userPrefs')))]"
```

```bash
dsc config get --file shallowMerge.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        timeout: 60
        retries: 3
        debug: true
messages: []
hadErrors: false
```

The final configuration shows `timeout` overridden by environment settings, `debug`
overridden by user preferences, and `retries` preserved from defaults.

### Example 3 - Shallow merge replaces nested objects

The following example demonstrates the key difference between shallow and deep merge. When a
property contains a nested object, the entire nested object is replaced rather than merged.

```yaml
# shallowMerge.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[shallowMerge(createArray(createObject('database', createObject('host', 'localhost', 'port', 5432, 'ssl', true())), createObject('database', createObject('host', 'prod.db.local'))))]"
```

```bash
dsc config get --file shallowMerge.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        database:
          host: prod.db.local
messages: []
hadErrors: false
```

The second object's `database` property completely replaces the first object's `database`,
losing the `port` and `ssl` properties. This is the shallow merge behavior.

### Example 4 - Merge with empty objects

The following example shows that empty objects in the array don't affect the merge result.

```yaml
# shallowMerge.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[shallowMerge(createArray(createObject('name', 'Service1', 'enabled', true()), createObject(), createObject('version', '2.0')))]"
```

```bash
dsc config get --file shallowMerge.example.4.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        name: Service1
        enabled: true
        version: '2.0'
messages: []
hadErrors: false
```

The empty object in the middle doesn't remove or affect any properties.

### Example 5 - Build feature flags from multiple sources

The following example merges base flags with team-specific and environment-specific overrides,
where each subsequent object overrides specific flags while preserving others.

```yaml
# shallowMerge.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[shallowMerge(createArray(createObject('newUI', false(), 'darkMode', true(), 'beta', false()), createObject('newUI', true()), createObject('beta', true())))]"
```

```bash
dsc config get --file shallowMerge.example.5.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        newUI: true
        darkMode: true
        beta: true
messages: []
hadErrors: false
```

Each subsequent object overrides specific flags while preserving others.

### Example 6 - Merge array results from parameters

The following example demonstrates merging parameter-based objects where the last occurrence
of each property wins, resulting in a single merged object.

```yaml
# shallowMerge.example.6.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  objects:
    type: array
    defaultValue:
    - name: alpha
      priority: 1
    - name: beta
      priority: 2
    - name: beta
      priority: 10
      critical: true
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[shallowMerge(parameters('objects'))]"
```

```bash
dsc config get --file shallowMerge.example.6.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        name: beta
        priority: 10
        critical: true
messages: []
hadErrors: false
```

The last occurrence of each property wins, resulting in a single merged object.

### Example 7 - Combine with objectKeys for validation

The following example uses `shallowMerge()` with [`objectKeys()`][01] and [`contains()`][05]
to validate that all expected configuration keys are present after merging.

```yaml
# shallowMerge.example.7.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  baseConfig:
    type: object
    defaultValue:
      timeout: 30
      retries: 3
  overrides:
    type: object
    defaultValue:
      timeout: 60
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      merged: "[shallowMerge(createArray(parameters('baseConfig'), parameters('overrides')))]"
      keys: "[objectKeys(shallowMerge(createArray(parameters('baseConfig'), parameters('overrides'))))]"
      hasRetries: "[contains(objectKeys(shallowMerge(createArray(parameters('baseConfig'), parameters('overrides')))), 'retries')]"
```

```bash
dsc config get --file shallowMerge.example.7.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        merged:
          timeout: 60
          retries: 3
        keys:
        - timeout
        - retries
        hasRetries: true
messages: []
hadErrors: false
```

This pattern ensures the merged configuration includes required properties.

### Example 8 - Empty array returns empty object

The following example shows that `shallowMerge()` returns an empty object when given an
empty array. It uses [`objectKeys()`][01], [`length()`][06], and [`equals()`][07] to verify
the result is empty.

```yaml
# shallowMerge.example.8.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      result: "[shallowMerge(createArray())]"
      isEmpty: "[equals(length(objectKeys(shallowMerge(createArray()))), 0)]"
```

```bash
dsc config get --file shallowMerge.example.8.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        result: {}
        isEmpty: true
messages: []
hadErrors: false
```

## Parameters

### inputArray

An array of objects to merge. Each element in the array should be an object. Non-object
elements in the array are silently ignored during the merge process.

```yaml
Type:     array
Required: true
Position: 1
```

## Output

Returns a single object containing all properties from the input objects. When the same
property appears in multiple objects, the value from the last object in the array is used.

```yaml
Type: object
```

## Error conditions

The function will return an error in the following cases:

- **Not an array**: The input is not an array (e.g., object, string, number, null)

## Notes

- This is a **shallow merge** - nested objects are replaced, not merged recursively
- Properties from objects later in the array override properties from earlier objects
- Empty objects in the array don't affect the merge
- Non-object elements in the array are ignored
- An empty array returns an empty object
- The function processes objects in array order, so the last object has highest precedence
- For recursive/deep merging of nested objects, consider using [`union()`][00] instead

## Related functions

- [`union()`][00] - Combines arrays or performs deep merge of objects
- [`createArray()`][02] - Creates an array from values
- [`createObject()`][03] - Creates an object from key-value pairs
- [`objectKeys()`][01] - Returns an array of keys from an object
- [`items()`][04] - Converts an object to an array of key-value pairs
- [`contains()`][05] - Checks if an array contains a specific value
- [`length()`][06] - Returns the number of elements in an array
- [`equals()`][07] - Compares two values for equality

<!-- Link reference definitions -->
[00]: ./union.md
[01]: ./objectKeys.md
[02]: ./createArray.md
[03]: ./createObject.md
[04]: ./items.md
[05]: ./contains.md
[06]: ./length.md
[07]: ./equals.md
