---
description: Reference for the 'skip' DSC configuration document function
ms.date:     08/29/2025
ms.topic:    reference
title:       skip
---

## Synopsis

Returns an array with all the elements after the specified number in the array,
or returns a string with all the characters after the specified number in the
string.

## Syntax

```Syntax
skip(<originalValue>, <numberToSkip>)
```

## Description

The `skip()` function returns the tail of an array or string by skipping a
specified number of items from the start.

- For arrays: returns a new array containing elements after the specified index
- For strings: returns a new string containing characters after the specified index

Both parameters are required. `originalValue` must be an array or a string.
`numberToSkip` must be an integer; negative values are treated as zero. If the
number is greater than the length of the array or string, the function returns
an empty array or an empty string respectively.

## Examples

### Example 1 - Skip elements in an array

The following example returns the tail of an array by skipping the first two
elements.

```yaml
# skip.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Tail of array
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      tail: "[skip(createArray('a','b','c','d'), 2)]"
```

```bash
dsc config get --file skip.example.1.dsc.config.yaml
```

```yaml
results:
- name: Tail of array
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        tail:
        - c
        - d
messages: []
hadErrors: false
```

### Example 2 - Skip characters in a string

The following example returns the substring of a string by skipping the first
two characters.

```yaml
# skip.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Tail of string
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      tail: "[skip('hello', 2)]"
```

```bash
dsc config get --file skip.example.2.dsc.config.yaml
```

```yaml
results:
- name: Tail of string
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        tail: llo
messages: []
hadErrors: false
```

## Parameters

### originalValue

The value to skip items from. Can be an array or a string.

```yaml
Type:     array | string
Required: true
Position: 1
```

### numberToSkip

The number of items to skip from the start. Must be an integer. Negative values
are treated as zero.

```yaml
Type:     int
Required: true
Position: 2
```

## Output

Returns the same type as `originalValue`:

- If `originalValue` is an array, returns an array
- If `originalValue` is a string, returns a string

```yaml
Type: array | string
```

## Error conditions

- `originalValue` is not an array or string
- `numberToSkip` is not an integer

## Related functions

- [`first()`][00] - Returns the first element or character
- [`length()`][01] - Returns the number of elements or characters

<!-- Link reference definitions -->
[00]: ./first.md
[01]: ./length.md
