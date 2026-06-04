---
description: Reference for the 'path' DSC configuration document function
ms.date:     06/04/2025
ms.topic:    reference
title:       path
---

# path

## Synopsis

Construct a file system path from one or more path segments

## Syntax

```Syntax
path(<path>, <child_path>, ...)
```

## Description

The `path()` function takes a base path and any number of child items to combine
into a single path, accounting for duplicate `/` characters. 

## Examples

### Example 1 - Construct with child path

This configuration constructs a simple absolute path of two elements. 

```yaml
# parseChildPath.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
    - name: Simple Path Construct
      type: Microsoft.DSC.Debug/Echo
      properties:
      output: "[path('C:\\Program Files', 'WindowsPowerShell')]"
```

```bash
dsc config get --file parseChildPath.example.1.dsc.config.yaml
```

```yaml
results:
- name: Simple Path Construct
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: C:\Program Files\WindowsPowerShell
messages: []
hadErrors: false
```

### Example 2 - Relative path with multiple elements

This configuration constructs a simple relative path of three elements. 

```yaml
# relativePath.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
    - name: Relative Path
      type: Microsoft.DSC.Debug/Echo
      properties:
      output: "[path('.\\usr', 'bin', 'bash')]"
```

```bash
dsc config get --file relativePath.example.2.dsc.config.yaml
```

```yaml
results:
- name: Relative Path
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: .\usr\bin\bash
messages: []
hadErrors: false
```

### Example 3 - Relative element in path

This configuration constructs a path with a double dot in that path. 

The path is returned as-is and is not resolved to an absolute path.

```yaml
# doubleDot.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
    - name: Double Dot Path
      type: Microsoft.DSC.Debug/Echo
      properties:
      output: "[path('parent', '..', 'child')]"
```

```bash
dsc config get --file doubleDot.example.3.dsc.config.yaml
```

```yaml
results:
- name: Double Dot Path
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: parent\..\child
messages: []
hadErrors: false
```

## Parameters

### path

The `path()` function expects at least two arguments, a base path and at 
least one child. 

The base path can be an absolute (e.g., `C:\Windows\System32`, `\usr\bin`), relative (e.g., `./System32`) or 
Universal Naming Convention (UNC) (e.g., `\\server1\c$\Windows`).

```yaml
Type:         string
Required:     true
Position:     1
```
### child

The `path()` function expects at least one child path to be supplied.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 18446744073709551615
```

## Output

Returns the concatenated path, made from the provided elements.

There are slight differences in return value depending on the OS (e.g., path separators, drive letters).

```yaml
Type: string
```

## Errors

The function returns an error in the following cases:

- **Invalid type**: Any argument is not a string

## Related functions

- [`join()`][01] - Joins an array into a single string, separated using a delimiter.
- [`uri()`][02] - Creates an absolute URI by combining the baseUri and the relativeUri string.

<!-- Link reference definitions -->
[01]: ./join.md
[02]: ./uri.md