---
title: 'Known issues: Microsoft Desired State Configuration'
description: "Troubleshooting and known issues for Microsoft Desired State Configuration (DSC)."
author: michaeltlombardi
ms.author: mlombardi
ms.service: dsc
ms.topic: troubleshooting-known-issue
ms.date: 07/14/2025
---

# Known issues: Microsoft Desired State Configuration

This article lists known issues and troubleshooting guidance for Microsoft Desired State Configuration (DSC).

The following table lists known issues with Microsoft DSC v3:

|                                                         Issue                                                         | Description                                                                     | Status    | Reported on                                          |
| :-------------------------------------------------------------------------------------------------------------------: | :------------------------------------------------------------------------------ | :-------- | :--------------------------------------------------- |
|               [Unable to parse content from `<manifestUrl>`](#unable-to-parse-content-from-manifesturl)               | When authoring a resource manifest in VSCode, you may encounter parsing errors. | Confirmed | [#917](https://github.com/PowerShell/DSC/issues/917) |
| [Resource not found when using Windows PowerShell adapter](#resource-not-found-when-using-windows-powershell-adapter) | A resource cannot be found when running DSC configuration using WinPS adapter.  | Confirmed | [#765](https://github.com/PowerShell/DSC/issues/765) |

For the most up-to-date information on known issues, visit the [DSC GitHub repository issues page](https://github.com/PowerShell/DSC/issues).

## Unable to parse content from `<manifestUrl>`

When authoring a resource manifest in Visual Studio Code (VSCode), you may encounter a parsing error:

> Unable to parse content from `<manifestUrl>`

This error occurs because canonical schema bundling is not fully supported in the 2020-12 JSON
schema specification. It applies to Microsoft DSC v3.0 and above.

### Prerequisites

- Visual Studio Code
- Microsoft DSC v3.0 or later

### Troubleshooting steps

To resolve this issue, use `manifest.vscode.json` in the schema URI for your resource manifest.
This enables enhanced authoring support in VSCode.

For more information, see [Enhanced authoring][00].

### Possible causes

- The resource manifest references a schema that is not compatible with the VSCode JSON schema parser.
- The canonical schema bundling feature is not yet supported in the 2020-12 JSON schema version
  used by VSCode.

## Resource not found when using Windows PowerShell adapter

When running DSC configurations with the Windows PowerShell (WinPS) adapter,
you may encounter errors indicating that a required resource cannot be found.

### Prerequisites

- Windows PowerShell DSC (PSDSC) 1.1 (included with Windows)
- DSC configuration using the WinPS adapter

### Issue description

The WinPS adapter relies on PSDSC 1.1, which uses the Local Configuration Manager (LCM) running
as a Windows service. By design, the LCM service only accesses resources installed for "AllUsers"
under the Program Files directory. If a DSC resource is installed for the current user only,
the service cannot detect or use it, resulting in a "resource not found" error.

This limitation is specific to PSDSC 1.1. PSDSC v2 addresses this issue, but it is not
included with Windows and requires separate installation.

### Troubleshooting steps

- Ensure all DSC resources required by your configuration are installed for "AllUsers" scope.
- Reinstall any missing resources using an elevated prompt to guarantee system-wide availability.

### Possible causes

- DSC resources installed only for the current user, not for all users.
- Using PSDSC 1.1, which restricts resource visibility to the "AllUsers" scope.

### Recommendation

Install all DSC resources, whether script-based and binary resources, for all users
("AllUsers" scope) to ensure they are available for the WinPS adapter.

## See also

- [Microsoft Desired State Configuration overview](../overview.md)

<!-- Link references -->
[00]: https://learn.microsoft.com/en-us/powershell/dsc/concepts/enhanced-authoring?view=dsc-3.0
