---
description: Reference for the 'first' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       first
---

# first

## Synopsis

Returns the first element of an array or the first character of a string.

## Syntax

```Syntax
first(<arrayOrString>)
```

## Description

The `first()` function extracts the first element from an array or the first
character from a string. When used with arrays, it returns the actual first
element preserving its original data type (string, number, array, or object).
When used with strings, it returns a new single-character string containing
the first Unicode character.

This function is particularly useful for accessing the primary or default item
from a collection, or extracting prefixes from identifiers and codes. The
function will return an error if the input array or string is empty.

## Examples

### Example 1 - Get the first server hostname

This example shows how to extract the primary server from a list of servers,
which could be useful for identifying the lead server in a cluster or getting
the default target for deployment operations.

```yaml
# first.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  servers:
    type: array
    defaultValue:
    - web01
    - web02
resources:
- name: First Server
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      firstServer: "[first(parameters('servers'))]"
```

```bash
dsc config get --file first.example.1.dsc.config.yaml
```

```yaml
results:
- name: First Server
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        firstServer: web01
messages: []
hadErrors: false
```

### Example 2 - Extract leading character for a prefix

This example demonstrates extracting the first character from an environment
code to create abbreviated prefixes for resource naming or tagging schemes.

```yaml
# first.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  environmentCode:
    type: string
    defaultValue: Prod
resources:
- name: Prefix Builder
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      prefix: "[first(parameters('environmentCode'))]"
```

```bash
dsc config get --file first.example.2.dsc.config.yaml
```

```yaml
results:
- name: Prefix Builder
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        prefix: P
messages: []
hadErrors: false
```

### Example 3 - Chain with array construction

This example shows how `first()` can be combined with `array()` to get the
first element from a dynamically constructed array, by wrapping a single
generated collection.

```yaml
# first.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Chained Example
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      firstMixed: "[first(array(createArray('a','b')))]"
```

```bash
dsc config get --file first.example.3.dsc.config.yaml
```

```yaml
results:
- name: Chained Example
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
  firstMixed: a
messages: []
hadErrors: false
```

## Parameters

### input

The array or string to extract the first element or character from.

```yaml
Type:     array or string
Required: true
```

If the input is an array, the first element is returned with its original data
type preserved. If the input is a string, the first Unicode character is
returned as a new string. Empty arrays and empty strings will result in an
error.

## Output

Returns the first element or character.

```yaml
Type: string | number | array | object
```

## Related functions

- [`array()`][00] - Creates an array from heterogeneous elements
- [`createArray()`][01] - Creates a homogeneous array
- [`indexOf()`][02] - Finds the index of an item in an array

<!-- Link reference definitions -->
[00]: ./array.md
[01]: ./createArray.md
[02]: ./indexOf.md
