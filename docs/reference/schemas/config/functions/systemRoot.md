--
description: Reference for the 'systemRoot' DSC configuration document function
ms.date:     06/04/2025
ms.topic:    reference
title:       systemRoot
---

# systemRoot

## Synopsis

Returns the system root path.

## Syntax

```Syntax
systemRoot()
```

## Description

The `systemRoot()` function returns the value of the system root path[01].

## Examples

### Example 1 - Get the current system root

The configuration uses the `systemRoot()` function to echo the current system root.

```yaml
# systemRoot.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo system root
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[systemRoot()]"
```

```bash
dsc config get --file systemRoot.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo system root
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: C:\
messages: []
hadErrors: false
```

### Example 2 - Construct and override the system root path

The configuration uses the `path()`[02] function to construct a path pf the `systemRoot()`, which 
is overriden using in the command line.

```yaml
# joinSystemRoot.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo system home
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[path(systemRoot(), 'home')]"
```

```bash
dsc config ---system-root / get --file joinSystemRoot.example.2.dsc.config.yaml
```

```yaml
results:
- name: Echo system home
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: /home
messages: []
hadErrors: false
```

## Output

The `systemRoot()` function returns the system root of the current host, or the value overriden
using the `--system-root` command line flag[01].

This is usually `C:\` on Windows system and `/` on Unix systems.

```yaml
Type: [string, int, bool, object, array]
```

<!-- Link reference definitions -->
[01]: ../../../cli/config/index.md#-r---system-root
[02]: ./path.md