---
description: Reference for the 'tryWhich' DSC configuration document function
ms.date:     11/19/2025
ms.topic:    reference
title:       tryWhich
---

## Synopsis

Looks for an executable in the `PATH` environment variable and returns the full path to the first  
matching executable or null if not found.

## Syntax

```Syntax
tryWhich(<commandName>)
```

## Description

The `tryWhich()` function searches for an executable in the `PATH` environment variable and returns  
the full path to the first matching executable if found. If the executable isn't discoverable, the  
function returns `null` instead of generating an error.

This function is useful for:

- Checking whether a required command-line tool is available before invoking it.  
- Conditionally configuring resources based on available system tools.  
- Validating prerequisites in configurations.  
- Finding the exact path to executables for use in scripts or commands.

The function searches the `PATH` in the same way the operating system would when executing a  
command. On Windows, it automatically checks for common executable extensions, like `.exe`, `.cmd`,  
and `.bat`, if no extension is provided.

Unlike a strict path lookup that would fail if the executable is missing, `tryWhich()`
gracefully returns `null`, making it ideal for conditional logic with [`if()`][00] or
[`coalesce()`][01].

## Examples

### Example 1 - Check if tool exists before using it

The following example uses `tryWhich()` with [`if()`][00] to conditionally set a property
based on whether the `git` command is available.

```yaml
# tryWhich.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      gitPath: "[tryWhich('git')]"
      hasGit: >-  
        [if(  
          equals(tryWhich('git'), null()),  
          false(),  
          true()  
        )]
```

```bash
dsc config get --file tryWhich.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        gitPath: /usr/bin/git
        hasGit: true
messages: []
hadErrors: false
```

If `git` wasn't discoverable in the `PATH` environmental variable, `gitPath` would be `null` and `hasGit`
would be `false`.

### Example 2 - Provide fallback paths with coalesce

The following example uses `tryWhich()` with [`coalesce()`][01] to provide fallback options
when searching for an executable, returning the first non-null path found.

```yaml
# tryWhich.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      pythonPath: >-  
        [coalesce(  
          tryWhich('python3'),  
          tryWhich('python'),  
          '/usr/bin/python'  
        )]
```

```bash
dsc config get --file tryWhich.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        pythonPath: /usr/bin/python3
messages: []
hadErrors: false
```

In this example, the function first looks for `python3` in the `PATH` environmental variable. If  
that executable isn't discovered, it then looks for `python`. If neither executable is discovered,  
it falls back to the specified default value, `/usr/bin/python3`.

### Example 3 - Validate multiple prerequisites

The following example demonstrates checking for multiple required tools and building a status
report using [`createObject()`][02].

```yaml
# tryWhich.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      prerequisites:
        docker: "[tryWhich('docker')]"
        kubectl: "[tryWhich('kubectl')]"
        helm: "[tryWhich('helm')]"
      allFound: >-  
        [and(  
          not(equals(tryWhich('docker'), null())),  
          not(equals(tryWhich('kubectl'), null())),  
          not(equals(tryWhich('helm'), null()))  
        )]
```

```bash
dsc config get --file tryWhich.example.3.dsc.config.yaml
```

```yaml
results:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        prerequisites:
          docker: /usr/bin/docker
          kubectl: /usr/local/bin/kubectl
          helm: null
        allFound: false
messages: []
hadErrors: false
```

This checks for three tools and determines if all are available. In this example, `helm` is
not found, so `allFound` is `false`.

## Parameters

### commandName

The name of the executable to locate. On Windows, it automatically checks for common executable  
extensions, like `.exe`, `.cmd`, and `.bat`, if no extension is provided.

```yaml
Type:     string
Required: true
Position: 1
```

## Output

Returns the full path to the first matching executable as a string if found in the system PATH.  
Returns `null` if the executable is not found.

```yaml
Type: string or null
```

## Error conditions

The function returns `null` instead of generating errors when the executable isn't found.  

The function only returns an error when the input isn't a string.

## Notes

- The function searches the `PATH` environment variable in the same order as the operating system.  
- On Windows, the function automatically checks for the executable with common extensions, like  
  `.exe`, `.cmd`, and `.bat`, when the input string doesn't define an extension. For example, if  
  the input is `dsc`, the function would return `dsc.exe` if available in `PATH`.  
- The function returns `null` when the executable isn't found instead of raising an error.  
- The function always returns the absolute path to a discovered executable.  
- Use with [`if()`][00] or [`coalesce()`][01] for conditional logic based on tool availability.  
- The function searches for the executable case-insensitively on Windows and case-sensitively on  
  other platforms.  
- The function resolves symbolic links to their target paths.

## Related functions

- [`if()`][00] - Conditional expression for checking if a tool exists
- [`coalesce()`][01] - Returns the first non-null value from a list
- [`equals()`][03] - Compares values for equality
- [`null()`][04] - Returns a null value
- [`and()`][05] - Logical AND for checking multiple conditions
- [`not()`][06] - Logical NOT for negating conditions
- [`createObject()`][02] - Creates an object from key-value pairs

<!-- Link reference definitions -->
[00]: ./if.md
[01]: ./coalesce.md
[02]: ./createObject.md
[03]: ./equals.md
[04]: ./null.md
[05]: ./and.md
[06]: ./not.md
