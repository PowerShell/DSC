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
value. Placeholders are specified with curly braces around the zero-based index of the argument.

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

### Example 2 - Format a string with parameters

The configuration uses other functions within the `format()` function to build a dynamic message.

```yaml
# format.example.2.dsc.config.yaml
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
      output: "[format('Hello, {0}! The time is {1}:{2}.', parameters('username'), parameters('hour'), parameters('minute'))]"
```

```bash
dsc --file format.example.2.dsc.config.yaml config get
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.0185508S
  name: Echo dynamic formatted string
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
Placeholders use the format `{n}`, where `n` is the zero-based index of the argument to insert.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

### arguments

The function accepts one or more arguments of any type to insert into the formatted string.

```yaml
Type:         any
Required:     true
MinimumCount: 1
MaximumCount: unlimited
```

## Output

The `format()` function returns a string where each placeholder in the **formatString** is replaced
with the corresponding argument value.

```yaml
Type: string
```
