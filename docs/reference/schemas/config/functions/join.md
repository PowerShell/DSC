---
description: Reference for the 'join' DSC configuration document function
ms.date:     08/29/2025
ms.topic:    reference
title:       join
---

## Synopsis

Joins an array into a single string, separated using a delimiter.

## Syntax

```Syntax
join(inputArray, delimiter)
```

## Description

The `join()` function takes an array and a delimiter.

- Each array element is converted to a string and concatenated with the
  delimiter between elements.

The `delimiter` can be any value; it’s converted to a string.

## Examples

### Example 1 - Produce a list of servers

Create a comma-separated string from a list of host names to pass to tools or
APIs that accept CSV input. This example uses [`createArray()`][02] to build
the server list and joins with ", ".

```yaml
# join.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[join(createArray('web01','web02','web03'), ', ')]"
```

```bash
dsc config get --file join.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: web01, web02, web03
messages: []
hadErrors: false
```

### Example 2 - Build a file system path from segments

Join path segments into a single path string. This is useful when composing
paths dynamically from parts.

```yaml
# join.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[join(createArray('/etc','nginx','sites-enabled'), '/')]"
```

```bash
dsc config get --file join.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: /etc/nginx/sites-enabled
messages: []
hadErrors: false
```

### Example 3 - Format a version string from numeric parts

Convert version components (numbers) into a dotted version string. Non-string
elements are converted to strings automatically.

```yaml
# join.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[join(createArray(1,2,3), '.')]"
```

```bash
dsc config get --file join.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: 1.2.3
messages: []
hadErrors: false
```

## Parameters

### inputArray

The array whose elements will be concatenated.

```yaml
Type:     array
Required: true
Position: 1
```

### delimiter

Any value used between elements. Converted to a string.

```yaml
Type:     any
Required: true
Position: 2
```

## Output

Returns a string containing the joined result.

```yaml
Type: string
```

## Related functions

- [`concat()`][00] - Concatenates strings together
- [`string()`][01] - Converts values to strings

<!-- Link reference definitions -->
[00]: ./concat.md
[01]: ./string.md
