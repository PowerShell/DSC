---
description: Reference for available functions in a Desired State Configuration document.
ms.date:     01/17/2024
ms.topic:    reference
title:       DSC Configuration document functions reference
---

# DSC Configuration document functions reference

## Synopsis

Functions available within a configuration document for runtime processing.

## Description

DSC configuration documents support the use of functions that DSC processes at runtime to determine
values for the document. These functions enable you to define configurations that reuse values and
are easier to maintain.

For DSC to recognize a function, it must be placed within square brackets in a string. DSC
configuration document functions use the following syntax:

```Syntax
[<function-name>(<function-parameters>...)]
```

When using functions in YAML, you must specify the function with a string value that is wrapped in
double quotation marks or uses the [folded][01] or [literal][02] block syntax. When using the
folded or literal block syntaxes, always use the [block chomping indicator][03] (`-`) to trim
trailing line breaks and empty lines.

```yaml
# Double quoted syntax
<keyword>: "[<function-name>(<function-parameters>...)]"
# Folded block syntax
<keyword>: >-
  [<function-name>(<function-parameters>...)]
# Literal block syntax
<keyword>: |-
  [<function-name>(<function-parameters>...)]
```

You can nest functions, using the output of a nested function as a parameter value for an outer
function. DSC processes nested functions from the innermost function to outermost function.

```Syntax
[<outer-function-name>(<nested-function-name>(<nested-function-parameters>))]
```

It can be difficult to read long functions, especially when they're deeply nested. You can use
newlines to break long functions into a more readable format with the folded or literal block
syntax.

```yaml
# Multi-line folded block syntax
<keyword>: >-
  [<outer-function-name>(
    <nested-function-name>(
      <deeply-nested-function-name>(<deeply-nested-function-parameters>)
    )
  )]
# Multi-line literal block syntax
<keyword>: |-
  [<outer-function-name>(
    <nested-function-name>(
      <deeply-nested-function-name>(<deeply-nested-function-parameters>)
    )
  )]
```

## Examples

### Example 1 - Use a function with valid syntaxes

The following configuration document shows the three valid syntaxes for specifying a function in
a configuration document. In each resource instance, the `text` property is set to the output of
the [base64()][base64] function.

```yaml
# overview.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
  - name: Double quoted syntax
    type: Test/Echo
    properties:
      text: "[base64('ab')]"
  - name: Folded block syntax
    type: Test/Echo
    properties:
      text: >-
        [base64('ab')]
  - name: Literal block syntax
    type: Test/Echo
    properties:
      text: |-
        [base64('ab')]
```

```sh
dsc --input-file overview.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: Double quoted syntax
  type: Test/Echo
  result:
    actualState:
      text: YWI=
- name: Folded block syntax
  type: Test/Echo
  result:
    actualState:
      text: YWI=
- name: Literal block syntax
  type: Test/Echo
  result:
    actualState:
      text: YWI=
messages: []
hadErrors: false
```

### Example 2 - Concatenate two strings

The following configuration document sets the `text` property of the resource instance to the
output of the [concat()][concat] function, combining the strings `a` and `b` into `ab`.

```yaml
# overview.example.2.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
  - name: Echo the concatenated strings 'a' and 'b'
    type: Test/Echo
    properties:
      text: "[concat('a', 'b')]"
```

```sh
dsc --input-file overview.example.2.dsc.config.yaml config get
```

```yaml
results:
- name: Echo the concatenated strings 'a' and 'b'
  type: Test/Echo
  result:
    actualState:
      text: ab
messages: []
hadErrors: false
```

### Example 3 - Using nested functions

The following configuration document shows how you can nest functions. The first two resource
instances use the output of the [concat()][concat] function as input to the [base64()][base64] function.
The third resource instance uses the output of the nested functions from the first two instances
as input to the `concat()` function. The last resource instance converts the output of the deeply
nested functions shown in the third instance to base64.

```yaml
# overview.example.3.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
  - name: Echo the concatenated strings 'a' and 'b' as base64
    type: Test/Echo
    properties:
      text: "[base64(concat('a', 'b'))]"
  - name: Echo the concatenated strings 'c' and 'd' as base64
    type: Test/Echo
    properties:
      text: "[base64(concat('c', 'd'))]"
  - name: Echo the concatenated base64 of strings 'ab' and 'cd'
    type: Test/Echo
    properties:
      text: "[concat(base64(concat('a', 'b')), base64(concat('c', 'd')))]"
  - name: Echo the concatenated base64 of strings 'ab' and 'cd' as base64
    type: Test/Echo
    properties:
      # text: "[base64(concat(base64(concat('a', 'b')), base64(concat('c', 'd'))))]"
      text: >-
        [base64(
          concat(
            base64(concat('a', 'b')),
            base64(concat('c', 'd'))
          )
        )]
```

```sh
dsc --input-file overview.example.3.dsc.config.yaml config get
```

```yaml
results:
- name: Echo the concatenated strings 'a' and 'b' as base64
  type: Test/Echo
  result:
    actualState:
      text: YWI=
- name: Echo the concatenated strings 'c' and 'd' as base64
  type: Test/Echo
  result:
    actualState:
      text: Y2Q=
- name: Echo the concatenated base64 of strings 'ab' and 'cd'
  type: Test/Echo
  result:
    actualState:
      text: YWI=Y2Q=
- name: Echo the concatenated base64 of strings 'ab' and 'cd' as base64
  type: Test/Echo
  result:
    actualState:
      text: WVdJPVkyUT0=
messages: []
hadErrors: false
```

## Functions

The following sections include the available DSC configuration functions by purpose and input type.

### Array functions

The following list of functions operate on arrays:

- [concat()][concat] - Combine multiple arrays of strings into a single array of strings.
- [createArray()][createArray] - Create an array of a given type from zero or more values of the
  same type.
- [min()][min] - Return the smallest integer value from an array of integers.
- [max()][max] - Return the largest integer value from an array of integers.

### Data functions

The following list of functions operate on data outside of a resource instance:

- [envvar()][envvar] - Return the value of a specified environment variable.
- [parameters()][parameters] - Return the value of a specified configuration parameter.

### Mathematics functions

The following list of functions operate on integer values or arrays of integer values:

- [add()][add] - Return the sum of two integers.
- [div()][div] - Return the dividend of two integers as an integer, dropping the remainder of the
  result, if any.
- [int()][int] - Convert a string or number with a fractional part into an integer.
- [max()][max] - Return the largest value from an array of integers.
- [min()][min] - Return the smallest value from an array of integers.
- [mod()][mod] - Return the remainder from the division of two integers.
- [mul()][mul] - Return the product from multiplying two integers.
- [sub()][sub] - Return the difference from subtracting one integer from another.

### Resource functions

The following list of functions operate on resource instances:

- [reference()][reference] - Return the result data for another resource instance.
- [resourceId()][resourceId] - Return the ID of another resource instance to reference or depend
  on.

### String functions

The following list of functions are for manipulating strings:

- [base64()][base64] - Return the base64 representation of a string.
- [concat()][concat] - Return a combined string where the input strings are concatenated in the
  order they're specified.

### Type functions

The following list of functions create or convert values of a given type:

- [createArray()][createArray] - Create an array of a given type from zero or more values of the
  same type.
- [int()][int] - Convert a string or number with a fractional part into an integer.

<!-- Link references -->
[01]: https://yaml.org/spec/1.2.2/#folded-style
[02]: https://yaml.org/spec/1.2.2/#literal-style
[03]: https://yaml.org/spec/1.2.2/#block-chomping-indicator
<!-- Function link references -->
[add]:         ./add.md
[base64]:      ./base64.md
[concat]:      ./concat.md
[createArray]: ./createArray.md
[div]:         ./div.md
[envvar]:      ./envvar.md
[int]:         ./int.md
[max]:         ./max.md
[min]:         ./min.md
[mod]:         ./mod.md
[mul]:         ./mul.md
[parameters]:  ./parameters.md
[reference]:   ./reference.md
[resourceId]:  ./resourceId.md
[sub]:         ./sub.md
