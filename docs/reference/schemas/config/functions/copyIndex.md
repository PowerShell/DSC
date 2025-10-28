---
description: Reference for the 'copyIndex' DSC configuration document function
ms.date:     02/28/2025
ms.topic:    reference
title:       copyIndex
---

# copyIndex

## Synopsis

Returns the current iteration index of a copy loop.

## Syntax

```Syntax
copyIndex()
copyIndex(<offset>)
copyIndex('<loopName>')
copyIndex('<loopName>', <offset>)
```

## Description

The `copyIndex()` function returns the current iteration index of a copy loop.
This function can only be used within resources that have a `copy` property
defined. The function is necessary for creating unique names and property
values for each instance created by the copy loop.

The index starts at 0 for the first iteration and increments by 1 for each
subsequent iteration. You can add an offset to shift the starting number, or
reference a specific loop by name when multiple copy loops are present.

## Examples

### Example 1 - Basic copyIndex usage

This example shows the basic usage of `copyIndex()` to create unique resource
names.

```yaml
# copyIndex.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Resource-{0}', copyIndex())]"
  copy:
    name: basicLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "Hello DSC"
```

```bash
dsc config get --file copyIndex.example.1.dsc.config.yaml
```

```yaml
results:
- name: Resource-0
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Hello World"
- name: Resource-1
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Hello World"
- name: Resource-2
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Hello World"
messages: []
hadErrors: false
```

### Example 2 - Using copyIndex with offset

This example demonstrates using an offset to start numbering from a different
value.

```yaml
# copyIndex.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Server-{0}', copyIndex(10))]"
  copy:
    name: serverLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "Server instance starting from 10 till 12"
```

```bash
dsc config get --file copyIndex.example.2.dsc.config.yaml
```

```yaml
results:
- name: Server-10
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Server instance starting from 10 till 12"
- name: Server-11
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Server instance starting from 10 till 12"
- name: Server-12
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Server instance starting from 10 till 12"
messages: []
hadErrors: false
```

### Example 3 - Using copyIndex with loop name

This example shows how to reference a specific loop by name.

```yaml
# copyIndex.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Item-{0}', copyIndex('itemLoop'))]"
  copy:
    name: itemLoop
    count: 1
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "Item from loop"
```

```bash
dsc config get --file copyIndex.example.3.dsc.config.yaml
```

```yaml
results:
- name: Item-0
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Item from loop"
messages: []
hadErrors: false
```

### Example 4 - Using copyIndex with loop name and offset

This example combines both loop name and offset parameters.

```yaml
# copyIndex.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Database-{0}', copyIndex('dbLoop', 100))]"
  copy:
    name: dbLoop
    count: 2
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "Database instance"
```

```bash
dsc config get --file copyIndex.example.4.dsc.config.yaml
```

```yaml
results:
- name: Database-100
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Database instance"
- name: Database-101
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Database instance"
messages: []
hadErrors: false
```

## Parameters

### offset

An optional integer offset to add to the current index. The offset must be a
non-negative number.

```yaml
Type:     integer
Required: false
Minimum:  0
```

### loopName

An optional string specifying the name of the copy loop to reference. This is
useful when you have multiple copy loops and need to reference a specific one.

```yaml
Type:     string
Required: false
```

## Output

The `copyIndex()` function returns an integer representing the current iteration
index, optionally adjusted by the offset.

```yaml
Type: integer
```

## Error Conditions

The `copyIndex()` function will return an error in the following situations:

- **Used outside copy loop**: The function can only be used within resources
  that have a `copy` property defined.
- **Negative offset**: The offset parameter must be non-negative.
- **Invalid loop name**: If a loop name is specified but no loop with that
  name exists.
- **Invalid arguments**: If the arguments provided are not of the expected
  types.

## Related Properties

- [`copy`][01] - Defines a loop to create multiple instances of a resource.

<!-- Link reference definitions -->
[01]: ./copy.md
