---
description: Reference for the 'lambda' DSC configuration document function
ms.date:     09/01/2026
ms.topic:    reference
title:       lambda
---

# lambda

## Synopsis

Creates a lambda function with named parameters and a body expression for use with functions that
evaluate an expression for each element of an array, like `map()` and `filter()`.

## Syntax

```Syntax
lambda(<elementName>, <body>)
lambda(<elementName>, <indexName>, <body>)
```

## Description

The `lambda()` function creates an anonymous function, called a _lambda_, that DSC evaluates once
for each element of an array. The result of `lambda()` is only usable as an argument to a function
that accepts lambdas. Currently, those functions are [`map()`][00] and [`filter()`][01].

A lambda consists of:

- One or more _parameter names_, specified as string literals. When `map()` or `filter()`
  evaluates the lambda for an element, DSC binds the element to the first parameter and the
  zero-based index of the element to the second parameter, if the lambda declares one.
- A _body_, specified as the last argument. The body must be an expression, which is a call to a
  configuration function, optionally followed by property or index access. A literal string,
  number, or boolean isn't a valid body.

DSC doesn't evaluate the body when it processes `lambda()`. Instead, DSC stores the parameter
names and the body expression and evaluates the body separately for every element when the
consuming function runs. Inside the body, use [`lambdaVariables()`][02] to read the value that DSC
bound to a parameter. The body can also call any other configuration function, including
[`parameters()`][03] and [`variables()`][04].

The `lambda()` function itself accepts any number of parameter names, but `map()` and `filter()`
only accept lambdas with one or two parameters. A lambda with more than two parameters raises an
error when one of those functions uses it.

## Examples

### Example 1 - Lambda with a single parameter

The following example creates a lambda with one parameter, `x`, and passes it to [`map()`][00].
For each element in the array, DSC binds the element to `x` and evaluates the body, which
multiplies the value by `10` with [`mul()`][05].

```yaml
# lambda.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(createArray(1, 2, 3), lambda('x', mul(lambdaVariables('x'), 10)))]"
```

```bash
dsc config get --file lambda.example.1.dsc.config.yaml
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
      - 30
messages: []
hadErrors: false
```

### Example 2 - Lambda with an element and index parameter

The following example creates a lambda with two parameters and passes it to [`filter()`][01].
DSC binds each element to `item` and the zero-based index of the element to `index`. The body
uses [`mod()`][06] and [`equals()`][07] to return `true` only for elements at even indexes.

```yaml
# lambda.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [filter(
        createArray('a', 'b', 'c', 'd'),
        lambda('item', 'index', equals(mod(lambdaVariables('index'), 2), 0))
      )]
```

```bash
dsc config get --file lambda.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - a
      - c
messages: []
hadErrors: false
```

In this example, the lambda declares the `item` parameter but only uses `index` in its body. The
first parameter is always bound to the element, so you must declare it even when the body only
needs the index.

### Example 3 - Lambda body that reads a configuration parameter

The following example shows that a lambda body can use other configuration functions. The body
calls [`parameters()`][03] to read the `prefix` parameter and [`concat()`][08] to combine it with
the current element.

```yaml
# lambda.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  prefix:
    type: string
    defaultValue: srv-
  names:
    type: array
    defaultValue: [web, db]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: >-
      [map(
        parameters('names'),
        lambda('name', concat(parameters('prefix'), lambdaVariables('name')))
      )]
```

```bash
dsc config get --file lambda.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
      - srv-web
      - srv-db
messages: []
hadErrors: false
```

## Parameters

### elementName

The name of the parameter that DSC binds to the current element of the array. The name must be a
string literal, like `'x'`. An expression that returns a string, like `string('x')`, isn't
accepted.

```yaml
Type:     string
Required: true
Position: 1
```

### indexName

The name of the parameter that DSC binds to the zero-based index of the current element. The name
must be a string literal. When you specify this parameter, the body must be the third argument.

```yaml
Type:     string
Required: false
Position: 2
```

### body

The expression to evaluate for each element. The body must be a call to a configuration function,
like `mul(lambdaVariables('x'), 2)`. Use [`lambdaVariables()`][02] inside the body to read the
values bound to the lambda's parameters.

The consuming function determines how it uses the value the body returns. [`map()`][00] collects
the returned values into the output array. [`filter()`][01] requires the body to return a boolean
value and keeps the element when the value is `true`.

The body is always the last argument. Its position is `2` when the lambda declares one parameter
and `3` when the lambda declares two parameters.

```yaml
Type:     expression
Required: true
Position: last
```

## Output

Returns a lambda value that can only be used as an argument to a function that accepts lambdas.
DSC represents the lambda as an opaque identifier string with the prefix `__lambda_`. If you use
`lambda()` where DSC doesn't expect a lambda, like directly as the value of a resource property,
the result is that identifier string, which isn't useful on its own.

```yaml
Type: lambda
```

## Error conditions

The function raises an error in the following cases:

- **Missing arguments**: The function is called with fewer than two arguments. DSC raises
  `lambda() requires at least one parameter name and a body expression`.
- **Parameter name isn't a string literal**: A parameter name is a number, a boolean, or an
  expression. DSC raises `lambda() parameter names must be string literals`.
- **Body isn't an expression**: The last argument is a literal value instead of a function call.
  DSC raises `lambda() body must be an expression`.
- **Passed to a function that doesn't accept lambdas**: The lambda is used as an argument to a
  function other than `map()` or `filter()`. For example, `concat('a', lambda('x', ...))` raises
  `Function 'concat' does not accept lambda arguments, accepted types are: String, Array`.
- **Too many parameters**: The lambda declares more than two parameters and is passed to `map()`
  or `filter()`. For example, DSC raises
  `Function 'map' requires lambda with 1 or 2 parameters (element and optional index)`.

## Notes

- The output of `dsc function list lambda` reports `minArgs` and `maxArgs` as `0` and an empty
  `acceptedArgOrderedTypes` list. This is because DSC's expression parser handles `lambda()`
  specially: it passes the arguments to the function without evaluating them first, which is how
  the body expression is captured instead of being evaluated immediately. The `constraints` field
  in the same output describes the actual requirement:
  `Lambda function must have at least one parameter and a body expression`.
- Parameter names are case-sensitive. The name you pass to [`lambdaVariables()`][02] must match
  the declared name exactly.
- Lambda parameters are separate from configuration variables and parameters. Declaring a lambda
  parameter with the same name as a configuration variable doesn't affect the variable, and
  [`variables()`][04] can't read a lambda parameter.
- DSC stores lambdas only for the duration of a single configuration evaluation.
- Only functions that declare a `lambda` argument type accept the result of `lambda()`. To see
  which argument types a function accepts, use `dsc function list <name>`.

## Related functions

- [`map()`][00] - Transforms every element of an array with a lambda
- [`filter()`][01] - Keeps only the elements of an array for which a lambda returns `true`
- [`lambdaVariables()`][02] - Reads a lambda parameter inside the lambda body
- [`parameters()`][03] - Returns the value of a configuration parameter
- [`variables()`][04] - Returns the value of a configuration variable
- [`mul()`][05] - Multiplies two integers
- [`mod()`][06] - Returns the remainder of dividing two integers
- [`equals()`][07] - Compares two values for equality
- [`concat()`][08] - Combines strings or arrays
- [`createArray()`][09] - Creates an array from values

<!-- Link reference definitions -->
[00]: ./map.md
[01]: ./filter.md
[02]: ./lambdaVariables.md
[03]: ./parameters.md
[04]: ./variables.md
[05]: ./mul.md
[06]: ./mod.md
[07]: ./equals.md
[08]: ./concat.md
[09]: ./createArray.md
