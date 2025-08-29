---
description: Reference for the 'join' DSC configuration document function
ms.date:     08/29/2025
ms.topic:    reference
title:       join
---

## Synopsis

Joins a string array into a single string, separated using a delimiter.

## Syntax

```Syntax
join(inputArray, delimiter)
```

## Description

The `join()` function takes either an array or a string and a delimiter.

- If `inputArray` is an array, each element is converted to a string and
  concatenated with the delimiter between elements.
- If `inputArray` is a string, its characters are joined with the delimiter
  between each character.

The `delimiter` can be any value; it is converted to a string.

## Examples

### Example 1 - Join array of strings

```yaml
# join.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[join(createArray('a','b','c'), '-')]"
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
      output: a-b-c
messages: []
hadErrors: false
```

### Example 2 - Join characters of a string

```yaml
# join.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[join('abc', '-')]"
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
      output: a-b-c
messages: []
hadErrors: false
```

## Parameters

### inputArray

An array or a string.

```yaml
Type:     array | string
Required: true
Position: 1
```

### delimiter

Any value used between elements/characters. Converted to a string.

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
