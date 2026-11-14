---
description: Query one process or export matching processes on Windows, Linux, and macOS.
ms.date: 11/09/2026
ms.topic: reference
title: Query and export running processes
---

# Query and export running processes

## Get the current PowerShell process

Run these commands in PowerShell on any supported platform:

```powershell
@{ pid = $PID } | ConvertTo-Json -Compress |
    dsc resource get -r Microsoft.System/Process -f -
```

The `actualState` contains the PID, name, and individual command-line arguments.
For example:

```yaml
actualState:
  pid: 1234
  name: pwsh
  cmdline:
    - pwsh
```

## Export processes using a name wildcard

```powershell
'{"name":"pwsh*"}' | dsc resource export -r Microsoft.System/Process -f -
```

The output is a configuration containing one instance per matching process. An unmatched name
produces an empty `resources` array without an error.

Export without filters:

```powershell
dsc resource export -r Microsoft.System/Process
```

Combine filters to select only the current PowerShell process:

```powershell
@{ pid = $PID; name = 'pwsh*' } | ConvertTo-Json -Compress |
    dsc resource export -r Microsoft.System/Process -f -
```
