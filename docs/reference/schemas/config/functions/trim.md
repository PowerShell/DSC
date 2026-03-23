---
description: Reference for the 'trim' DSC configuration document function
ms.date:     01/10/2025
ms.topic:    reference
title:       trim
---

# trim

## Synopsis

Removes all leading and trailing white-space characters from the specified string.

## Syntax

```Syntax
trim(<stringToTrim>)
```

## Description

The `trim()` function removes all leading and trailing white-space characters from
the input string. White-space characters include spaces, tabs, newlines, carriage
returns, and other Unicode whitespace characters. The function preserves internal
whitespace within the string. Use it for cleaning user input, normalizing
configuration values, or preparing strings for comparison.

## Examples

### Example 1 - Clean user input

The following example removes leading and trailing spaces from a parameter value.

```yaml
# trim.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  userName:
    type: string
    defaultValue: '  admin  '
resources:
- name: Clean user input
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      rawInput: "[parameters('userName')]"
      cleanedInput: "[trim(parameters('userName'))]"
```

```bash
dsc config get --file trim.example.1.dsc.config.yaml
```

```yaml
results:
- name: Clean user input
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        rawInput: '  admin  '
        cleanedInput: admin
messages: []
hadErrors: false
```

### Example 2 - Normalize file paths

The following example demonstrates using `trim()` with [`concat()`][01] to clean
path components before building a complete file path.

```yaml
# trim.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  baseDir:
    type: string
    defaultValue: '  /var/log  '
  fileName:
    type: string
    defaultValue: '  app.log  '
resources:
- name: Build clean file path
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      filePath: "[concat(trim(parameters('baseDir')), '/', trim(parameters('fileName')))]"
```

```bash
dsc config get --file trim.example.2.dsc.config.yaml
```

```yaml
results:
- name: Build clean file path
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        filePath: /var/log/app.log
messages: []
hadErrors: false
```

### Example 3 - Clean configuration values for comparison

The following example uses `trim()` to normalize strings before comparing them with
the [`equals()`][02] function.

```yaml
# trim.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  expectedEnv:
    type: string
    defaultValue: production
  actualEnv:
    type: string
    defaultValue: '  production  '
resources:
- name: Environment comparison
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      matches: "[equals(trim(parameters('actualEnv')), parameters('expectedEnv'))]"
```

```bash
dsc config get --file trim.example.3.dsc.config.yaml
```

```yaml
results:
- name: Environment comparison
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        matches: true
messages: []
hadErrors: false
```

### Example 4 - Process multi-line configuration

The following example shows how `trim()` handles tabs, newlines, and various
whitespace characters.

```yaml
# trim.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Whitespace handling
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      spaces: "[trim('   content   ')]"
      mixed: "[trim(' \t\n  content  \n\t ')]"
      internal: "[trim('  multiple  spaces  inside  ')]"
```

```bash
dsc config get --file trim.example.4.dsc.config.yaml
```

```yaml
results:
- name: Whitespace handling
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        spaces: content
        mixed: content
        internal: multiple  spaces  inside
messages: []
hadErrors: false
```

### Example 5 - Combine with case conversion

The following example demonstrates using `trim()` with [`toLower()`][00] to both
clean and normalize a string value.

```yaml
# trim.example.5.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  serviceName:
    type: string
    defaultValue: '  WEB-SERVER  '
resources:
- name: Clean and normalize service name
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      original: "[parameters('serviceName')]"
      normalized: "[toLower(trim(parameters('serviceName')))]"
```

```bash
dsc config get --file trim.example.5.dsc.config.yaml
```

```yaml
results:
- name: Clean and normalize service name
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        original: '  WEB-SERVER  '
        normalized: web-server
messages: []
hadErrors: false
```

## Parameters

### stringToTrim

The string value to remove leading and trailing whitespace from.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

The `trim()` function returns the input string with all leading and trailing
white-space characters removed. Internal whitespace is preserved.

```yaml
Type: string
```

## Related functions

- [`toLower()`][00] - Converts a string to lower case
- [`concat()`][01] - Concatenates strings together
- [`equals()`][02] - Compares two values for equality
- [`startsWith()`][03] - Checks if a string starts with a value
- [`endsWith()`][04] - Checks if a string ends with a value
- [`substring()`][05] - Extracts a portion of a string
- [`replace()`][06] - Replaces text in a string
- [`parameters()`][07] - Retrieves parameter values

<!-- Link reference definitions -->
[00]: ./toLower.md
[01]: ./concat.md
[02]: ./equals.md
[03]: ./startsWith.md
[04]: ./endsWith.md
[05]: ./substring.md
[06]: ./replace.md
[07]: ./parameters.md
