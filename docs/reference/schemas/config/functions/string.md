---
description: Reference for the 'string' DSC configuration document function
ms.date:     08/09/2025
ms.topic:    reference
title:       string
---

# string

## Synopsis

Converts a value to a string representation.

## Syntax

```Syntax
string(<value>)
```

## Description

The `string()` function converts a value of any type to its string
representation. This is useful for formatting output, concatenating values, or
ensuring consistent data types. Arrays and objects are converted to JSON
strings, while primitive types are converted to their standard string
representations.

## Examples

### Example 1 - Convert integers to strings

The following example shows how to convert numbers to strings for display
purposes.

```yaml
# string.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  serverCount:
    type: int
    defaultValue: 42
  memorySize:
    type: int
    defaultValue: 16
resources:
- name: Convert numbers
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      serverCountString: "[string(parameters('serverCount'))]"
      memorySizeString: "[string(parameters('memorySize'))]"
      literalNumber: "[string(123)]"
```

```bash
dsc config get --file string.example.1.dsc.config.yaml
```

```yaml
results:
- name: Convert numbers
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        serverCountString: '42'
        memorySizeString: '16'
        literalNumber: '123'
messages: []
hadErrors: false
```

### Example 2 - Convert arrays and objects to JSON strings

The following example shows how arrays and objects are converted to JSON
strings.

```yaml
# string.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  serverList:
    type: array
    defaultValue:
    - web01
    - web02
    - db01
  config:
    type: object
    defaultValue:
      timeout: 30
      retries: 3
      enabled: true
resources:
- name: Convert collections
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      serversJson: "[string(parameters('serverList'))]"
      configJson: "[string(parameters('config'))]"
      arrayLiteral: "[string(createArray('a', 'b'))]"
```

```bash
dsc config get --file string.example.3.dsc.config.yaml
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.1452881S
  name: Convert collections
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        serversJson: '["web01","web02","db01"]'
        configJson: '{"timeout":30,"retries":3,"enabled":true}'
        arrayLiteral: '["a","b"]'
messages: []
hadErrors: false
```

### Example 3 - Building formatted messages

The following example shows a practical use case for building formatted
messages using string conversion.

```yaml
# string.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  deploymentId:
    type: int
    defaultValue: 12345
  isProduction:
    type: bool
    defaultValue: false
  serverCount:
    type: int
    defaultValue: 3
resources:
- name: Build status message
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      deploymentInfo: "[concat('Deployment ', string(parameters('deploymentId')), ' running in ', if(parameters('isProduction'), 'production', 'development'), ' mode')]"
      serverMessage: "[concat('Managing ', string(parameters('serverCount')), ' server(s)')]"
      statusFlag: "[concat('Production: ', string(parameters('isProduction')))]"
```

```bash
dsc config get --file string.example.4.dsc.config.yaml
```

```yaml
results:
- name: Build status message
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        deploymentInfo: Deployment 12345 running in development mode
        serverMessage: Managing 3 server(s)
        statusFlag: 'Production: false'
messages: []
hadErrors: false
```

### Example 4 - String conversion for logging

The following example demonstrates converting various data types for logging
purposes.

```yaml
# string.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  timestamp:
    type: int
    defaultValue: 1691596800
  errorCode:
    type: int
    defaultValue: 404
  metadata:
    type: object
    defaultValue:
      source: "api"
      level: "error"
resources:
- name: Generate log entry
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      logEntry: "[concat('[', string(parameters('timestamp')), '] ERROR ', string(parameters('errorCode')), ': ', string(parameters('metadata')))]"
```

```bash
dsc config get --file string.example.5.dsc.config.yaml
```

```yaml
results:
- name: Generate log entry
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        logEntry: '[1691596800] ERROR 404: {"level":"error","source":"api"}'
messages: []
hadErrors: false
```

## Parameters

### value

The value to convert to a string.

```yaml
Type:         [string, number, bool, null, array, object]
Required:     true
```

The `string()` function accepts exactly one input value of any type. The
conversion behavior depends on the input type:

- **String**: Returns the string unchanged
- **Number**: Converts to decimal string representation
- **Boolean**: Converts to "true" or "false"
- **Null**: Converts to "null"
- **Array**: Converts to JSON array string
- **Object**: Converts to JSON object string

## Output

The `string()` function returns the string representation of the input value.

```yaml
Type: string
```

<!-- Link reference definitions -->
