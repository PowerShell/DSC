---
description: Reference for the 'map' DSC configuration document function
ms.date:     09/01/2026
ms.topic:    reference
title:       map
---

# map

## Synopsis

Transforms an array by applying a lambda function to each element and returning the results as a
new array.

## Syntax

```Syntax
map(<inputArray>, <lambda>)
```

## Description

The `map()` function evaluates a lambda function created with [`lambda()`][00] against every
element of an array and returns a new array that contains the value the lambda returned for each
element. The output array always has the same number of elements as the input array, in the same
order.

For each element in the input array, DSC:

1. Binds the element to the lambda's first parameter.
1. Binds the zero-based index of the element to the lambda's second parameter, if the lambda
   declares one.
1. Evaluates the lambda's body expression. Inside the body, use [`lambdaVariables()`][01] to read
   the bound parameters.
1. Appends the value the body returned to the output array.

Unlike [`filter()`][02], the lambda body can return a value of any type. The returned values
don't need to have the same type as the input elements, so you can use `map()` to convert an array
of numbers into an array of strings, an array of objects into an array of one of their properties,
or an array of values into an array of objects.

The lambda body can call any other configuration function. DSC evaluates the body with a copy of
the current context, so the body can read configuration [`parameters()`][03] and
[`variables()`][04] in addition to the lambda's own parameters.

This function is useful for:

- Applying the same calculation or formatting to every element of an array.
- Extracting a single property from every object in an array.
- Building an array of objects from an array of simple values.
- Generating sequential names or values together with [`range()`][05].

## Examples

### Example 1 - Multiply every element

The following example multiplies every number in the `numbers` parameter by `2`. The lambda
declares a single parameter, `x`, which DSC binds to each element in turn.

```yaml
# map.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  numbers:
    type: array
    defaultValue: [1, 2, 3]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(parameters('numbers'), lambda('x', mul(lambdaVariables('x'), 2)))]"
```

```bash
dsc config get --file map.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - 2
      - 4
      - 6
messages: []
hadErrors: false
```

### Example 2 - Use the element index

The following example uses a lambda with two parameters. DSC binds the element to `val` and the
zero-based index of the element to `i`. The lambda adds the two values together.

```yaml
# map.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  items:
    type: array
    defaultValue: [10, 20, 30]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [map(
        parameters('items'),
        lambda('val', 'i', add(lambdaVariables('val'), lambdaVariables('i')))
      )]
```

```bash
dsc config get --file map.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - 10
      - 21
      - 32
messages: []
hadErrors: false
```

### Example 3 - Generate names from a range

The following example combines `map()` with [`range()`][05] and [`format()`][06] to generate a
sequence of server names. The input elements are numbers and the output elements are strings.

```yaml
# map.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(range(1, 3), lambda('n', format('server-{0}', lambdaVariables('n'))))]"
```

```bash
dsc config get --file map.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - server-1
      - server-2
      - server-3
messages: []
hadErrors: false
```

### Example 4 - Extract and reshape object properties

The following example maps an array of objects twice. The `names` output extracts the `name`
property of every object with the property access syntax. The `summary` output uses
[`createObject()`][07] to build a new object for every element from its index and its properties.

```yaml
# map.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  users:
    type: array
    defaultValue:
    - name: alice
      role: admin
    - name: bob
      role: user
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      names: "[map(parameters('users'), lambda('user', lambdaVariables('user').name))]"
      summary: >-
        [map(
          parameters('users'),
          lambda(
            'user',
            'index',
            createObject(
              'id', lambdaVariables('index'),
              'label', format(
                '{0} ({1})',
                lambdaVariables('user').name,
                lambdaVariables('user').role
              )
            )
          )
        )]
```

```bash
dsc config get --file map.example.4.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        names:
        - alice
        - bob
        summary:
        - id: 0
          label: alice (admin)
        - id: 1
          label: bob (user)
messages: []
hadErrors: false
```

## Parameters

### inputArray

The array whose elements to transform. The elements can be of any type, including objects and
nested arrays.

```yaml
Type:     array
Required: true
Position: 1
```

### lambda

A lambda function created with [`lambda()`][00]. The lambda must declare one or two parameters.
DSC binds the current element to the first parameter and the zero-based index of the element to
the optional second parameter. The value the lambda's body returns becomes the corresponding
element of the output array.

```yaml
Type:     lambda
Required: true
Position: 2
```

## Output

Returns an array with one element for each element of `inputArray`, in the same order. Each
element is the value the lambda returned for the corresponding input element. Returns an empty
array when `inputArray` is empty.

```yaml
Type: array
```

## Error conditions

The function raises an error in the following cases:

- **Not an array**: The first argument isn't an array. For example, passing a string raises
  `Function 'map' does not accept string arguments, accepted types are: Array`.
- **Not a lambda**: The second argument isn't a lambda created with [`lambda()`][00].
- **Too many parameters**: The lambda declares more than two parameters. DSC raises
  `Function 'map' requires lambda with 1 or 2 parameters (element and optional index)`.
- **Body error**: The lambda body raises an error. For example, calling
  [`lambdaVariables()`][01] with a name that the lambda didn't declare raises
  `Lambda parameter '<name>' not found in current context`.

## Notes

- The function always returns an array with the same length as the input array. To remove
  elements from an array, use [`filter()`][02] instead.
- The function evaluates the lambda body once for each element in the input array. When the input
  array is empty, the body is never evaluated and the function returns an empty array.
- The index that DSC binds to the optional second parameter is a zero-based number.
- The lambda body can return a value of any type, including objects and arrays.
- Lambda parameters are separate from configuration variables. Read them with
  [`lambdaVariables()`][01], not [`variables()`][04].

## Related functions

- [`lambda()`][00] - Creates the lambda function that `map()` evaluates
- [`lambdaVariables()`][01] - Reads a lambda parameter inside the lambda body
- [`filter()`][02] - Keeps only the elements of an array for which a lambda returns `true`
- [`parameters()`][03] - Returns the value of a configuration parameter
- [`variables()`][04] - Returns the value of a configuration variable
- [`range()`][05] - Creates an array of sequential integers
- [`format()`][06] - Creates a formatted string from input values
- [`createObject()`][07] - Creates an object from key-value pairs
- [`mul()`][08] - Multiplies two integers
- [`add()`][09] - Adds two integers

<!-- Link reference definitions -->
[00]: ./lambda.md
[01]: ./lambdaVariables.md
[02]: ./filter.md
[03]: ./parameters.md
[04]: ./variables.md
[05]: ./range.md
[06]: ./format.md
[07]: ./createObject.md
[08]: ./mul.md
[09]: ./add.md
