---
description: Reference for the 'int' DSC configuration document function
ms.date:     04/09/2024
ms.topic:    reference
title:       int
---

# int

## Synopsis

Returns an integer from an input string or non-integer number.

## Syntax

```Syntax
int(<inputValue>)
```

## Description

The `int()` function returns an integer, converting an input string or non-integer number into an
integer. It truncates the fractional part of the input number, it doesn't round up.

## Examples

### Example 1 - Create an integer from a string

This configuration returns an integer, converting the string value `'4.7'` to `4`.

```yaml
# int.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
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

The `int()` function expects input as either a string or a number. If the value is a string that
can't be parsed as a number, DSC returns an error for the function.

> [!NOTE]
> There is an open bug (see [GitHub issue #390][#390]) for this function when operating on numbers.
> The function correctly returns the expected value for string representations of numbers with
> fractional parts, but returns the fractional part instead of the integer for actual numbers.
> Specify the input value for this function as a string instead of a number.

```yaml
Type:         [String, Number]
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
[#390]: https://github.com/PowerShell/DSC/issues/390
