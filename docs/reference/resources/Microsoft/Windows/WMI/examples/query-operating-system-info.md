---
description: >
    Example showing how to use the Microsoft.Windows/WMI resource adapter to query
    system information using the Win32_ComputerSystem class.

ms.date: 03/25/2025
ms.topic: reference
title: Query system information using WMI adapter
---

This example demonstrates how to use the `Microsoft.Windows/WMI` resource adapter 
to query basic system information from a computer using the Win32_ComputerSystem WMI class.

## List available system properties

First, you can discover the available properties for the Win32_ComputerSystem class by running:

```powershell
dsc resource list --adapter Microsoft.Windows/WMI
```

To list out only one WMI class, you can run the follwoing command:

```powershell
dsc resource list --adapter Microsoft.Windows/WMI root.cimv2/Win32_ComputerSystem
```

DSC returns the following information:

```text
Type                             Kind      Version  Capabilities  RequireAdapter         Description
----------------------------------------------------------------------------------------------------
root.cimv2/Win32_ComputerSystem  Resource           gs--t---      Microsoft.Windows/WMI
```

## Query the operating system info

To retrieve basic system information, the following snippets shows how you can use the resource
with [dsc resource get][01] command:

```powershell
dsc resource get --resource root.cimv2/Win32_ComputerSystem
```

This command returns a JSON object containing information about the computer system.

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/get.md
