---
description: Reference for the 'lambdaVariables' DSC configuration document function
ms.date:     09/01/2026
ms.topic:    reference
title:       lambdaVariables
---

# lambdaVariables

## Synopsis

Returns the value bound to a named parameter of the lambda function that DSC is currently
evaluating.

## Syntax

```Syntax
lambdaVariables(<name>)
```

## Description

The `lambdaVariables()` function retrieves the value of a lambda parameter. It's only meaningful
inside the body of a lambda created with [`lambda()`][00]. When [`map()`][01] or [`filter()`][02]
evaluates the lambda for an element, DSC binds the element to the lambda's first parameter and the
zero-based index of the element to the optional second parameter. The `lambdaVariables()` function
returns those bound values by name.

The returned value has whatever type the bound value has. The element can be of any type, and the
index is always a number. When the returned value is an object or an array, you can use the
property and index access syntax on the result, like `lambdaVariables('server').name` or
`lambdaVariables('server').ports[0]`.

Lambda parameters are separate from configuration variables and parameters. The
[`variables()`][03] function can't read a lambda parameter, and `lambdaVariables()` can't read a
configuration variable.

If the name doesn't match a parameter of the lambda that DSC is currently evaluating, including
when you use `lambdaVariables()` outside of a lambda body, DSC raises an error and stops
processing the configuration document.

## Examples

### Example 1 - Read the current element

The following example uses `lambdaVariables()` to read the element that DSC bound to the `x`
parameter and adds `1` to it with [`add()`][04].

```yaml
# lambdaVariables.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(createArray(1, 2, 3), lambda('x', add(lambdaVariables('x'), 1)))]"
```

```bash
dsc config get --file lambdaVariables.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - 2
      - 3
      - 4
messages: []
hadErrors: false
```

### Example 2 - Read the element and its index

The following example declares two lambda parameters. DSC binds each element to `color` and the
zero-based index of the element to `position`. The body reads both values with
`lambdaVariables()` and combines them into an object with [`createObject()`][05].

```yaml
# lambdaVariables.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [map(
        createArray('red', 'green', 'blue'),
        lambda(
          'color',
          'position',
          createObject(
            'position', lambdaVariables('position'),
            'color', lambdaVariables('color')
          )
        )
      )]
```

```bash
dsc config get --file lambdaVariables.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - position: 0
        color: red
      - position: 1
        color: green
      - position: 2
        color: blue
messages: []
hadErrors: false
```

### Example 3 - Access properties of an object element

The following example maps an array of objects. The body uses the property access syntax on the
result of `lambdaVariables()` to read the `name` property and the first item of the `ports` array
for each element, then combines them with [`format()`][06].

```yaml
# lambdaVariables.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  servers:
    type: array
    defaultValue:
    - name: web01
      ports: [80, 443]
    - name: db01
      ports: [5432]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [map(
        parameters('servers'),
        lambda(
          'server',
          format(
            '{0}:{1}',
            lambdaVariables('server').name,
            lambdaVariables('server').ports[0]
          )
        )
      )]
```

```bash
dsc config get --file lambdaVariables.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - web01:80
      - db01:5432
messages: []
hadErrors: false
```

## Parameters

### name

The name of the lambda parameter to read. The name is case-sensitive and must exactly match one of
the parameter names declared in the enclosing [`lambda()`][00] call.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

Returns the value that DSC bound to the named parameter for the current element. For the first
lambda parameter, the value is the element itself and can be of any type. For the optional second
lambda parameter, the value is the zero-based index of the element as a number.

```yaml
Type: [array, boolean, null, number, object, string]
```

## Error conditions

The function raises an error in the following cases:

- **Unknown parameter**: The name doesn't match a parameter of the lambda that DSC is currently
  evaluating. DSC raises `Lambda parameter '<name>' not found in current context`.
- **Used outside a lambda**: The function is called outside of a lambda body, so no lambda
  parameters are bound. DSC raises the same
  `Lambda parameter '<name>' not found in current context` error.
- **Invalid name**: The argument isn't a string.

## Notes

- Parameter names are case-sensitive.
- The index bound to the optional second lambda parameter is a zero-based number.
- DSC binds the lambda parameters fresh for every element. The body can't read the values bound
  for other elements of the array.
- Lambda parameters are separate from the values returned by [`variables()`][03] and
  [`parameters()`][07]. You can still call those functions inside a lambda body to read
  configuration variables and parameters.

## Related functions

- [`lambda()`][00] - Creates a lambda function with named parameters
- [`map()`][01] - Transforms every element of an array with a lambda
- [`filter()`][02] - Keeps only the elements of an array for which a lambda returns `true`
- [`variables()`][03] - Returns the value of a configuration variable
- [`parameters()`][07] - Returns the value of a configuration parameter
- [`add()`][04] - Adds two integers
- [`createObject()`][05] - Creates an object from key-value pairs
- [`format()`][06] - Creates a formatted string from input values

<!-- Link reference definitions -->
[00]: ./lambda.md
[01]: ./map.md
[02]: ./filter.md
[03]: ./variables.md
[04]: ./add.md
[05]: ./createObject.md
[06]: ./format.md
[07]: ./parameters.md
