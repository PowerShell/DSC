---
description: Reference for the 'filter' DSC configuration document function
ms.date:     09/01/2026
ms.topic:    reference
title:       filter
---

# filter

## Synopsis

Returns a new array that contains only the elements of an input array for which a lambda function
returns `true`.

## Syntax

```Syntax
filter(<inputArray>, <lambda>)
```

## Description

The `filter()` function evaluates a lambda function created with [`lambda()`][00] against every
element of an array and returns a new array that contains only the elements for which the lambda
returned `true`. The function doesn't change the elements it keeps. It copies them to the output
array in their original order.

For each element in the input array, DSC:

1. Binds the element to the lambda's first parameter.
1. Binds the zero-based index of the element to the lambda's second parameter, if the lambda
   declares one.
1. Evaluates the lambda's body expression. Inside the body, use [`lambdaVariables()`][01] to read
   the bound parameters.
1. Includes the element in the output when the body returns `true` and skips it when the body
   returns `false`.

The lambda body must return a boolean value. If it returns a value of any other type, DSC raises
an error and stops processing the configuration document.

The lambda body can call any other configuration function. DSC evaluates the body with a copy of
the current context, so the body can read configuration [`parameters()`][02] and
[`variables()`][03] in addition to the lambda's own parameters.

This function is useful for:

- Selecting the subset of an array that meets a condition before passing it to a resource.
- Filtering an array of objects by the value of one of their properties.
- Keeping only specific positions of an array by testing the index parameter.

## Examples

### Example 1 - Filter numbers by value

The following example keeps only the numbers greater than `2`. The lambda declares a single
parameter, `x`, which DSC binds to each element in turn. The [`greater()`][04] function returns
the boolean value that `filter()` requires.

```yaml
# filter.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  numbers:
    type: array
    defaultValue: [1, 2, 3, 4, 5]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [filter(
        parameters('numbers'),
        lambda('x', greater(lambdaVariables('x'), 2))
      )]
```

```bash
dsc config get --file filter.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - 3
      - 4
      - 5
messages: []
hadErrors: false
```

### Example 2 - Filter elements by index

The following example uses a lambda with two parameters. DSC binds the element to `val` and the
zero-based index of the element to `i`. The lambda uses [`less()`][05] on the index to keep only
the first two elements.

```yaml
# filter.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  items:
    type: array
    defaultValue: [10, 20, 30, 40]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [filter(
        parameters('items'),
        lambda('val', 'i', less(lambdaVariables('i'), 2))
      )]
```

```bash
dsc config get --file filter.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - 10
      - 20
messages: []
hadErrors: false
```

### Example 3 - Filter objects by a property

The following example filters an array of objects. The lambda body accesses the `enabled`
property of each element with the property access syntax. Because the property is already a
boolean value, the lambda returns it directly.

```yaml
# filter.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  services:
    type: array
    defaultValue:
    - name: web
      enabled: true
    - name: database
      enabled: false
    - name: cache
      enabled: true
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [filter(
        parameters('services'),
        lambda('service', lambdaVariables('service').enabled)
      )]
```

```bash
dsc config get --file filter.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - name: web
        enabled: true
      - name: cache
        enabled: true
messages: []
hadErrors: false
```

The output contains the complete objects that matched, not just the property that was tested.

### Example 4 - Filter strings by prefix

The following example uses [`startsWith()`][06] in the lambda body to keep only the strings that
start with `prod-`.

```yaml
# filter.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [filter(
        createArray('prod-web', 'dev-web', 'prod-db', 'test-db'),
        lambda('name', startsWith(lambdaVariables('name'), 'prod-'))
      )]
```

```bash
dsc config get --file filter.example.4.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - prod-web
      - prod-db
messages: []
hadErrors: false
```

## Parameters

### inputArray

The array whose elements to test. The elements can be of any type, including objects and nested
arrays.

```yaml
Type:     array
Required: true
Position: 1
```

### lambda

A lambda function created with [`lambda()`][00]. The lambda must declare one or two parameters.
DSC binds the current element to the first parameter and the zero-based index of the element to
the optional second parameter. The lambda's body must return a boolean value.

```yaml
Type:     lambda
Required: true
Position: 2
```

## Output

Returns an array containing the elements of `inputArray` for which the lambda returned `true`, in
their original order. Returns an empty array when no elements match or when `inputArray` is empty.

```yaml
Type: array
```

## Error conditions

The function raises an error in the following cases:

- **Not an array**: The first argument isn't an array. For example, passing a string raises
  `Function 'filter' does not accept string arguments, accepted types are: Array`.
- **Not a lambda**: The second argument isn't a lambda created with [`lambda()`][00].
- **Too many parameters**: The lambda declares more than two parameters. DSC raises
  `Function 'filter' requires lambda with 1 or 2 parameters (element and optional index)`.
- **Non-boolean result**: The lambda body returns a value that isn't a boolean. DSC raises
  `filter() lambda must return a boolean value`.
- **Body error**: The lambda body raises an error. For example, calling
  [`lambdaVariables()`][01] with a name that the lambda didn't declare raises
  `Lambda parameter '<name>' not found in current context`.

## Notes

- The function doesn't transform the elements it keeps. To change the elements of an array, use
  [`map()`][07] instead.
- The function evaluates the lambda body once for each element in the input array. When the input
  array is empty, the body is never evaluated and the function returns an empty array.
- The index that DSC binds to the optional second parameter is a zero-based number.
- Lambda parameters are separate from configuration variables. Read them with
  [`lambdaVariables()`][01], not [`variables()`][03].

## Related functions

- [`lambda()`][00] - Creates the lambda function that `filter()` evaluates
- [`lambdaVariables()`][01] - Reads a lambda parameter inside the lambda body
- [`map()`][07] - Transforms every element of an array with a lambda
- [`parameters()`][02] - Returns the value of a configuration parameter
- [`variables()`][03] - Returns the value of a configuration variable
- [`greater()`][04] - Checks whether the first value is greater than the second value
- [`less()`][05] - Checks whether the first value is less than the second value
- [`startsWith()`][06] - Checks whether a string starts with a prefix
- [`createArray()`][08] - Creates an array from values

<!-- Link reference definitions -->
[00]: ./lambda.md
[01]: ./lambdaVariables.md
[02]: ./parameters.md
[03]: ./variables.md
[04]: ./greater.md
[05]: ./less.md
[06]: ./startsWith.md
[07]: ./map.md
[08]: ./createArray.md
