---
description: Reference for the 'toUpper' DSC configuration document function
ms.date:     10/03/2025
ms.topic:    reference
title:       toUpper
---

# toUpper

## Synopsis

Converts the specified string to upper case.

## Syntax

```Syntax
toUpper(<stringToChange>)
```

## Description

The `toUpper()` function converts all lowercase letters in the input string to
uppercase letters. Numbers, symbols, punctuation, and whitespace are unchanged.
The function supports Unicode characters and preserves the original string
structure. Use it for normalizing string comparisons, formatting output, or
ensuring consistent casing in configuration values.

## Examples

### Example 1 - Convert user input to uppercase

The following example converts a parameter value to uppercase for consistent
processing.

```yaml
# toupper.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  serviceName:
    type: string
    defaultValue: web-api-service
resources:
- name: Service name conversion
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      originalName: "[parameters('serviceName')]"
      upperName: "[toUpper(parameters('serviceName'))]"
```

```bash
dsc config get --file toupper.example.1.dsc.config.yaml
```

```yaml
results:
- name: Service name conversion
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        originalName: web-api-service
        upperName: WEB-API-SERVICE
messages: []
hadErrors: false
```

### Example 2 - Format configuration values

The following example demonstrates using `toUpper()` with the [`concat()`][01]
function to create formatted configuration keys.

```yaml
# toupper.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  environment:
    type: string
    defaultValue: production
  component:
    type: string
    defaultValue: database
resources:
- name: Configuration key formatting
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      configKey: "[concat(toUpper(parameters('environment')), '_', toUpper(parameters('component')), '_CONFIG')]"
```

```bash
dsc config get --file toupper.example.2.dsc.config.yaml
```

```yaml
results:
- name: Configuration key formatting
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        configKey: PRODUCTION_DATABASE_CONFIG
messages: []
hadErrors: false
```

### Example 3 - Conditional uppercase conversion

The following example uses `toUpper()` conditionally with the [`if()`][02]
function based on a parameter value.

```yaml
# toupper.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  text:
    type: string
    defaultValue: Hello World
  shouldCapitalize:
    type: bool
    defaultValue: true
resources:
- name: Conditional uppercase
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      result: "[if(parameters('shouldCapitalize'), toUpper(parameters('text')), parameters('text'))]"
```

```bash
dsc config get --file toupper.example.3.dsc.config.yaml
```

```yaml
results:
- name: Conditional uppercase
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        result: HELLO WORLD
messages: []
hadErrors: false
```

### Example 4 - Unicode and special character handling

The following example shows how `toUpper()` handles Unicode characters and
preserves special characters.

```yaml
# toupper.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Unicode and special character conversion
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      ascii: "[toUpper('hello world!')]"
      unicode: "[toUpper('café résumé')]"
      mixed: "[toUpper('Server-01 (primary)')]"
```

```bash
dsc config get --file toupper.example.4.dsc.config.yaml
```

```yaml
results:
- name: Unicode and special character conversion
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        ascii: HELLO WORLD!
        unicode: CAFÉ RÉSUMÉ
        mixed: SERVER-01 (PRIMARY)
messages: []
hadErrors: false
```

## Parameters

### stringToChange

The string value to convert to upper case.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

The `toUpper()` function returns the input string with all lowercase letters
converted to uppercase. Numbers, symbols, punctuation, and whitespace remain
unchanged.

```yaml
Type: string
```

## Related functions

- [`toLower()`][00] - Converts a string to lower case
- [`concat()`][01] - Concatenates strings together
- [`if()`][02] - Returns values based on a condition
- [`string()`][03] - Converts values to strings
- [`parameters()`][04] - Retrieves parameter values

<!-- Link reference definitions -->
[00]: ./toLower.md
[01]: ./concat.md
[02]: ./if.md
[03]: ./string.md
[04]: ./parameters.md
