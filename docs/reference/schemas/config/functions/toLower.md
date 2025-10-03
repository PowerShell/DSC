---
description: Reference for the 'toLower' DSC configuration document function
ms.date:     10/03/2025
ms.topic:    reference
title:       toLower
---

# toLower

## Synopsis

Converts the specified string to lower case.

## Syntax

```Syntax
toLower(<stringToChange>)
```

## Description

The `toLower()` function converts all uppercase letters in the input string to
lowercase letters. Numbers, symbols, punctuation, and whitespace are unchanged.
The function supports Unicode characters and preserves the original string
structure. Use it for normalizing string comparisons, creating consistent
naming conventions, or formatting output for case-sensitive operations.

## Examples

### Example 1 - Normalize resource names

The following example converts a parameter value to lowercase for consistent
resource naming.

```yaml
# tolower.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  resourceName:
    type: string
    defaultValue: WEB-API-SERVICE
resources:
- name: Resource name normalization
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      originalName: "[parameters('resourceName')]"
      normalizedName: "[toLower(parameters('resourceName'))]"
```

```bash
dsc config get --file tolower.example.1.dsc.config.yaml
```

```yaml
results:
- name: Resource name normalization
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        originalName: WEB-API-SERVICE
        normalizedName: web-api-service
messages: []
hadErrors: false
```

### Example 2 - Create consistent file paths

The following example demonstrates using `toLower()` with the [`concat()`][01]
function to create lowercase file paths.

```yaml
# tolower.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  fileName:
    type: string
    defaultValue: CONFIG-FILE
  extension:
    type: string
    defaultValue: JSON
resources:
- name: File path creation
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      filePath: "[concat('/etc/', toLower(parameters('fileName')), '.', toLower(parameters('extension')))]"
```

```bash
dsc config get --file tolower.example.2.dsc.config.yaml
```

```yaml
results:
- name: File path creation
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        filePath: /etc/config-file.json
messages: []
hadErrors: false
```

### Example 3 - Case-insensitive comparison preparation

The following example uses `toLower()` to normalize strings for comparison
using the `equals()` function.

```yaml
# tolower.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  userInput:
    type: string
    defaultValue: Production
  expectedValue:
    type: string
    defaultValue: PRODUCTION
resources:
- name: Case-insensitive comparison
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      matches: "[equals(toLower(parameters('userInput')), toLower(parameters('expectedValue')))]"
```

```bash
dsc config get --file tolower.example.3.dsc.config.yaml
```

```yaml
results:
- name: Case-insensitive comparison
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        matches: true
messages: []
hadErrors: false
```

### Example 4 - Unicode and special character handling

The following example shows how `toLower()` handles Unicode characters and
preserves special characters.

```yaml
# tolower.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Unicode and special character conversion
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      ascii: "[toLower('HELLO WORLD!')]"
      unicode: "[toLower('CAFÉ RÉSUMÉ')]"
      mixed: "[toLower('SERVER-01 (PRIMARY)')]"
```

```bash
dsc config get --file tolower.example.4.dsc.config.yaml
```

```yaml
results:
- name: Unicode and special character conversion
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        ascii: hello world!
        unicode: café résumé
        mixed: server-01 (primary)
messages: []
hadErrors: false
```

## Parameters

### stringToChange

The string value to convert to lower case.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

The `toLower()` function returns the input string with all uppercase letters
converted to lowercase. Numbers, symbols, punctuation, and whitespace remain
unchanged.

```yaml
Type: string
```

## Related functions

- [`toUpper()`][00] - Converts a string to upper case
- [`concat()`][01] - Concatenates strings together
- [`equals()`][02] - Compares two values for equality
- [`if()`][03] - Returns values based on a condition
- [`string()`][04] - Converts values to strings
- [`parameters()`][05] - Retrieves parameter values

<!-- Link reference definitions -->
[00]: ./toUpper.md
[01]: ./concat.md
[02]: ./equals.md
[03]: ./if.md
[04]: ./string.md
[05]: ./parameters.md
