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

Defines the base path that the function appends child path segments to. The base path must be a
string value. It can be any of the following kinds of paths:

- Absolute, like `C:\Windows\System32` or `/usr/bin`
- Relative, like `.\infrastructure` or `../compliance/pci`
- Universal Naming Convention (UNC), such as `\\server1\c$\Windows`

```yaml
Type:         string
Required:     true
Position:     1
```
### child

Defines the child path segments the function appends to the base path. The function expects at
least one child path segment. Every child path segment must be a string value.

The function appends each segment to the output path in the order that you specify them. The
function inserts the operating system's path separator (`\` on Windows, `/` on Linux and macOS)
between each defined segment unless the segment has a trailing forward slash (`/`).

> [!NOTE]
> On Windows systems, when you specify any absolute path as a child path segment, like `C:\dsc`,
> the function _replaces_ the currently constructed path with that absolute path segment.
>
> For example, `[path('./a', 'b', 'C:\', 'd')]` resolves to `C:\d` on Windows and `./a/b/C:\/d`
> on non-Windows systems.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 18446744073709551615
```

## Output

Returns the concatenated path, made from the provided elements.

The output path for the same input depends on the operating systems:

- The function uses the operating system's defined path separator for appending child path segments
  to the base path (`\` for Windows and `/` for Linux and macOS).
  
  For example, `[path('a', 'b', 'c')]` resolves to `a\b\c` on Windows and `a/b/c` on Linux and macOS.
- On Windows, specifying a child path segment that begins with a drive letter _replaces_ the
  constructed path instead of appending to it.
  
  For example, `[path('./a', 'b', 'C:\', 'd')]` resolves to `C:\d` on Windows and `./a/b/C:\/d`
  on non-Windows systems.

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