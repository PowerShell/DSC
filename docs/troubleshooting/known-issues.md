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

| Issue                                                                           | Description                                                                             |  Status   | Reported on  |
|:--------------------------------------------------------------------------------|:----------------------------------------------------------------------------------------|:---------:|:------------:|
| [Unable to parse content from `<manifestUrl>`](#t01)                            | When authoring a resource manifest in VS Code, you may encounter parsing errors.        | Confirmed | [#917][#917] |
| [Resource not found when using Windows PowerShell adapter](#t02)                | A resource can't be found when using the `Microsoft.Windows/WindowsPowerShell` adapter. | Confirmed | [#765][#765] |
| [Validation errors when executing dsc.exe in Windows PowerShell sessions](#t03) | DSC raises input validation errors when invoked in Windows PowerShell                   | Confirmed | [#965][#965] |
| [Numeric string values coerced to integers in Windows PowerShell](#t04)          | DSC raises schema errors for string-typed properties with numeric values in Windows PowerShell. | Confirmed | — |

For the most up-to-date information on known issues, see the [DSC GitHub repository issues][01].

## Unable to parse content from `<manifestUrl>`

<a id="t01"></a>

When authoring a resource manifest in Visual Studio Code (VS Code), you may encounter a parsing error:

> Unable to parse content from `<manifestUrl>`

This error occurs because canonical schema bundling isn't fully supported in VS Code. Canonical
schema bundling was introduced with the 2020-12 JSON schema specification.

### Prerequisites

- Visual Studio Code
- Microsoft DSC v3.0 or later

### Troubleshooting steps

To resolve this issue, use `manifest.vscode.json` in the schema URI for your resource manifest.
This enables enhanced authoring support in VS Code.

For more information, see [Enhanced authoring][02].

### Possible causes

- The resource manifest references a schema that isn't compatible with the VS Code JSON schema parser.
- VS Code doesn't currently support parsing canonically bundled schemas.

## Resource not found when using Windows PowerShell adapter

<a id="t02"></a>

When running DSC configurations with the `Microsoft.Windows/WindowsPowerShell` adapter, you may  
encounter errors indicating that a required resource cannot be found.

### Prerequisites

- Windows PowerShell DSC (PSDSC) 1.1 (included with Windows)
- Using the `Microsoft.Windows/WindowsPowerShell` adapter in a configuration document or to  
  directly invoke a resource

### Issue description

The [Microsoft.Windows/WindowsPowerShell][03] adapter relies on PSDSC 1.1, which uses the Local  
Configuration Manager (LCM) running as a Windows service. By design, the LCM service only accesses  
resources installed in the **AllUsers** scope under the Program Files directory. If a PSDSC module  
is installed for the current user only, the service cannot detect or use it, resulting in a  
"resource not found" error.  

This limitation is specific to the `Microsoft.Windows/WindowsPowerShell` adapter. It doesn't affect  
the `Microsoft.DSC/PowerShell` adapter, which doesn't rely on PSDSC 1.1.

### Troubleshooting steps

- Ensure all PSDSC modules required by your configuration are installed in the **AllUsers** scope.
- Reinstall the PowerShell module for any missing PSDSC resources using an elevated prompt to  
  guarantee system-wide availability.

### Possible causes

- A PSDSC resource module is installed only for the current user, not for all users.

### Recommendation

Install all PSDSC resource modules in the **AllUsers** scope to ensure they're available for the  
`Microsoft.Windows/WindowsPowerShell` adapter.

## Validation errors when executing dsc.exe in Windows PowerShell sessions

<a id="t03"></a>  

When executing `dsc.exe` commands in Windows PowerShell sessions, you may encounter
validation errors when using manually crafted JSON input or the `ConvertTo-Json` cmdlet
with the `-Compress` parameter. This issue is related to how Windows PowerShell handles
string encoding and JSON formatting.

### Prerequisites

- Windows PowerShell session
- Direct execution of `dsc.exe` commands
- Use of JSON input with the `--input` parameter

### Problem details

When running `dsc.exe` commands in Windows PowerShell, validation errors may occur
when passing JSON input to resources. This typically happens when using manually
crafted JSON strings or when using PowerShell's `ConvertTo-Json` cmdlet with the `-Compress` parameter.

Commands that work correctly:

- `dsc resource get -r PSDesiredStateConfiguration/Service --input '{ "Name": "bits" }'`
- `dsc resource get -r PSDesiredStateConfiguration/Service --input (@{Name = 'bits'} | ConvertTo-Json)`

Common error symptoms include:

- JSON parsing failures when using compressed JSON output.  
- Property validation errors with valid JSON input.  
- Cannot validate argument on parameter `<parameterName>`. The argument is null  
  or empty, or an element of the argument collection contains a null value.

### Resolution steps

Recommend piping JSON over stdin with the --file - syntax:

```powershell
@{ Name = 'bits' } |  
    ConvertTo-Json -Compress |  
    dsc resource get -r PSDesiredStateConfiguration/Service --file - 
```

### Root causes

- Windows PowerShell's handling of compressed JSON may introduce formatting issues.  
- String encoding differences between Windows PowerShell and `dsc.exe`.  
- JSON parsing inconsistencies when using the `-Compress` parameter with `ConvertTo-Json`.

### Recommendation

When executing `dsc.exe` commands in Windows PowerShell:

- Recommend piping JSON over stdin with the --file - syntax.
- Use `ConvertTo-Json` without the `-Compress` parameter.
- Consider using PowerShell 7+ for improved JSON handling compatibility.

## Numeric string values coerced to integers in Windows PowerShell

<a id="t04"></a>

When running DSC configuration commands in Windows PowerShell, YAML string values that look like
numbers — such as port numbers explicitly quoted as `"8080"` — are coerced to integers before
schema validation. DSC then fails to validate the integer value against the schema property typed
as `string`, even though the YAML source file is correctly authored.

This issue is closely related to [Validation errors when executing dsc.exe in Windows PowerShell
sessions](#t03). Both issues share the same root cause: Windows PowerShell's handling of values
differs from PowerShell 7.

### Error message

> ERROR Schema: 8080 is not of type "string"

### Prerequisites

- Windows PowerShell session
- A DSC configuration document or resource input that defines string-typed properties with
  values that look like integers. For example, port numbers such as `"8080"` or `"23"`

### Problem details

Some DSC resource schemas define properties as `string` even when their values are numeric.
Port numbers are a common example. YAML allows you to preserve the string type by quoting the
value explicitly (`localPorts: "8080"`). PowerShell 7 preserves that explicit type through to
schema validation.

In Windows PowerShell, the quoted numeric string is coerced to a JSON integer before DSC
validates it. The difference is visible in DSC's `Verify JSON` debug output:

**PowerShell 7 (correct):**

```json
"localPorts": "8080"
```

**Windows PowerShell (incorrect):**

```json
"localPorts": 8080
```

DSC validates `8080` (integer) against the schema property typed as `string` and fails.

### Resolution steps

Run `dsc` commands from a PowerShell 7 or later session. PowerShell 7 preserves the string type
of explicitly quoted numeric values in YAML, so schema validation succeeds.

If Windows PowerShell is required, pipe the configuration file over stdin using the `--file -`
syntax. This bypasses the coercion issue because DSC reads the YAML directly from the file stream
rather than through PowerShell's value handling:

```powershell
Get-Content .\firewall.config.dsc.yaml | dsc config test --file -
```

### Possible causes

- Windows PowerShell does not preserve the explicit string quoting for numeric-looking YAML
  values, coercing them to integers before DSC can validate them.

### Recommendation

Run all `dsc` commands from a PowerShell 7 or later session to avoid this coercion behavior.

## See also

- [Microsoft Desired State Configuration overview][04]

<!-- Link references -->  
[01]: https://github.com/PowerShell/DSC/issues  
[02]: ../concepts/enhanced-authoring.md  
[03]: ../reference/resources/Microsoft/Windows/WindowsPowerShell/index.md  
[04]: ../overview.md  
[#917]: https://github.com/PowerShell/DSC/issues/917  
[#765]: https://github.com/PowerShell/DSC/issues/765  
[#965]: https://github.com/PowerShell/DSC/issues/965  
