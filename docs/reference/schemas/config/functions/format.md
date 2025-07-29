---
description: Reference for the 'format' DSC configuration document function
ms.date:     02/28/2025
ms.topic:    reference
title:       format
---

# format

## Synopsis

Returns a formatted string that uses placeholders to insert values.

## Syntax

```Syntax
format(<formatString>, <arg1>, <arg2>, ...)
```

## Description

The `format()` function returns a string that includes formatted values using a template and
placeholders. Each placeholder in the template string is replaced with the corresponding argument
value.

### Placeholder syntax

Placeholders must be defined with the following syntax:

```Syntax
{<index>[:<formatSpecifier>]}
```

Every placeholder must specify the zero-based index of the argument.

This function supports a subset of format specifiers for controlling how data is formatted in the
string. To use a format specifier, define a placeholder followed by a colon and then a specifier.

The following table defines the supported format specifiers and the data types they're valid for.
If you use a format specifier with an invalid data type, DSC raises an error.

| Specifier | Format                | Valid Types |
|:---------:|:----------------------|:------------|
| `b`       | Binary                | `integer`   |
| `e`       | Lowercase exponential | `integer`   |
| `E`       | Uppercase exponential | `integer`   |
| `o`       | Octal                 | `integer`   |
| `e`       | Lowercase hexadecimal | `integer`   |
| `E`       | Uppercase hexadecimal | `integer`   |

## Examples

### Example 1 - Format a simple string

The configuration formats a string with two placeholders.

```yaml
# format.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo formatted string
    type: Microsoft.DSC.Debug/Echo
    properties:
        output: "[format('Hello, {0}! Today is {1}.', 'World', 'Monday')]"
```

```bash
dsc config get --file format.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo formatted string
    type: Microsoft.DSC.Debug/Echo
    result:
        actualState:
            output: Hello, World! Today is Monday.
messages: []
hadErrors: false
```

### Example 2 - Format specifiers

This example demonstrates formatting every supported data type and format specifier.

```yaml
# format.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: descriptive resource name
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      string:  "[format('{0} {1}', 'hello', 'world')]"
      boolean: "[format('{0} or {1}', true, false)]"
      integer:
        binary:            "[format('{0} => {0:b}', 123)]"
        octal:             "[format('{0} => {0:o}', 123)]"
        lowercaseHex:      "[format('{0} => {0:x}', 123)]"
        uppercaseHex:      "[format('{0} => {0:X}', 123)]"
        lowercaseExponent: "[format('{0} => {0:e}', 123)]"
        uppercaseExponent: "[format('{0} => {0:E}', 123)]"
```

```bash
dsc config get --file format.example.2.dsc.config.yaml
```

```yaml
results:
- name: descriptive resource name
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        string: hello world
        boolean: true or false
        integer:
          binary: 1111011
          octal: 173
          lowercaseHex: 7b
          uppercaseHex: 7B
          lowercaseExponent: 1.23e2
          uppercaseExponent: 1.23E2
messages: []
hadErrors: false
```

### Example 3 - Format a string with parameters

The configuration uses other functions within the `format()` function to build a dynamic message.

```yaml
# format.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  username:
    type: string
    defaultValue: Mikey
  hour:
    type: string
    defaultValue: "09"
  minute:
    type: string
    defaultValue: "30"
resources:
  - name: Echo dynamic formatted string
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: >-
        [format(
          'Hello, {0}! The time is {1}:{2}.',
          parameters('username'),
          parameters('hour'),
          parameters('minute')
        )]
```

```bash
dsc --file format.example.3.dsc.config.yaml config get
```

```yaml
results:
- name: Echo dynamic formatted string
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello, Mikey! The time is 09:30.
messages: []
hadErrors: false
```

## Parameters

### formatString

The `format()` function requires a template string that includes placeholders for argument values.

Placeholders use the zero-based index of the function arguments. You can reference the same
argument any number of times. If DSC can't resolve the placeholder index to an argument, DSC raises
an error.

For more information about the syntax for defining placeholders, see
[Placeholder syntax](#placeholder-syntax).

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### arguments

The function accepts one or more arguments to insert into the formatted string.

```yaml
Type:         [boolean, integer, string]
Required:     true
MinimumCount: 1
MaximumCount: 18446744073709551615
```

## Output

The `format()` function returns a string where each placeholder in the `formatString` is replaced
with the corresponding argument value.

```yaml
Type: string
```
