---
description: Reference for the 'int' DSC configuration document function
ms.date:     02/28/2025
ms.topic:    reference
title:       int
---

# int

## Synopsis

Returns an integer from an input string or integer.

## Syntax

```Syntax
int(<inputValue>)
```

## Description

The `int()` function returns an integer, converting an input string into an integer. If you pass an
integer, it returns the integer. If you pass any other value, including a non-integer number, the
function raises an invalid input error.

## Examples

### Example 1 - Create an integer from a string

This configuration returns an integer, converting the string value `'4.7'` to `4`.

```yaml
# int.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo integer value
  type: Test/Echo
  properties:
    output: "[int('4.7')]"
```

```bash
dsc config get --document int.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Echo integer value of '4.7'
  type: Test/Echo
  result:
    actualState:
      output: 4
messages: []
hadErrors: false
```

## Parameters

### inputValue

The `int()` function expects input as either a string or an integer. If the value is a string that
can't be parsed as a number DSC returns an error for the function. If the value isn't a string or
integer, DSC returns an invalid input error for the function.

```yaml
Type:         [String, Integer]
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `int()` function returns an integer representation of the input. If the input value is a string
or number with a fractional part, the function returns the integer without the fractional part. It
doesn't round up, so for an input value of `4.999` the function returns `4`.

```yaml
Type: integer
```

<!-- Link reference definitions -->
