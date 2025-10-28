---
description: Reference for the 'context' DSC configuration document function
ms.date:     09/26/2025
ms.topic:    reference
title:       context
---

# context

## Synopsis

Returns information about the current execution context including operating system
and security context.

## Syntax

```Syntax
context()
```

## Description

The `context()` function returns an object containing information about the current
execution context. This includes details about the operating system (family,
version, architecture, etc.) and the current security context (elevated,
current, or restricted). The function takes no parameters and always returns
the same structure with current system information.

The returned object contains:

- `os`: Operating system information including family, version, edition,
  architecture, and other OS details
- `security`: The current security context (Current, Elevated, or Restricted)

## Examples

### Example 1 - Basic context information

This example shows how to retrieve and display basic context information.

```yaml
# context.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Show context information
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[context()]"
```

```bash
dsc config get --file context.example.1.dsc.config.yaml
```

```yaml
results:
- name: Show context information
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        os:
          family: Windows
          version: "10.0.22631"
          edition: "Windows 11 Pro"
          bitness: 64
          architecture: "x86_64"
        security: Current
messages: []
hadErrors: false
```

### Example 2 - Conditional configuration based on elevation

This example demonstrates using the context function with [`if()`][00],
[`equals()`][01], and [`not()`][02] to conditionally configure resources based
on whether DSC is running with elevated privileges.

```yaml
# context.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Check if running elevated
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[if(equals(context().security, 'Elevated'), 'Running as Administrator', 'Running as Standard User')]"
  - name: Elevated-only configuration
    type: Microsoft.DSC.Debug/Echo
    condition: "[equals(context().security, 'Elevated')]"
    properties:
      output: "This resource only runs when elevated"
  - name: Standard user configuration
    type: Microsoft.DSC.Debug/Echo
    condition: "[not(equals(context().security, 'Elevated'))]"
    properties:
      output: "This resource runs for standard users"
```

```bash
# When running as standard user
dsc config get --file context.example.2.dsc.config.yaml
```

```yaml
results:
- name: Check if running elevated
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Running as Standard User"
- name: Standard user configuration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "This resource runs for standard users"
messages: []
hadErrors: false
```

### Example 3 - OS-specific configuration using context

This example shows how to use the context function with [`equals()`][01] and
[`concat()`][03] to create OS-specific configurations.

```yaml
# context.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Show OS information
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        family: "[context().os.family]"
        version: "[context().os.version]"
        architecture: "[context().os.architecture]"
  - name: Windows-specific resource
    type: Microsoft.DSC.Debug/Echo
    condition: "[equals(context().os.family, 'Windows')]"
    properties:
      output: "[concat('Running on Windows ', context().os.edition)]"
  - name: Linux-specific resource
    type: Microsoft.DSC.Debug/Echo
    condition: "[equals(context().os.family, 'Linux')]"
    properties:
      output: "[concat('Running on Linux ', context().os.version)]"
  - name: macOS-specific resource
    type: Microsoft.DSC.Debug/Echo
    condition: "[equals(context().os.family, 'macOS')]"
    properties:
      output: "[concat('Running on macOS ', context().os.version)]"
```

```bash
# When running on Windows
dsc config get --file context.example.3.dsc.config.yaml
```

```yaml
results:
- name: Show OS information
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        family: "Windows"
        version: "10.0.22631"
        architecture: "x86_64"
- name: Windows-specific resource
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Running on Windows Windows 11 Pro"
messages: []
hadErrors: false
```

## Parameters

The `context()` function takes no parameters.

```yaml
Type:         N/A
Required:     false
MinimumCount: 0
MaximumCount: 0
```

## Output

The `context()` function returns an object with the following structure:

```yaml
Type: object
```

### os

The `os` property contains detailed information about the operating system:

- `family`: The operating system family (Windows, Linux, or macOS)
- `version`: The version string of the operating system
- `edition`: The edition of the operating system (e.g., "Windows 11 Pro",
  "Ubuntu 22.04 LTS")
- `codename`: The codename for the operating system (primarily for Linux
  distributions)
- `bitness`: The bitness of the operating system (32 or 64)
- `architecture`: The processor architecture (e.g., "x86_64", "arm64")

### security

The `security` property indicates the current security context:

- `Current`: Running with current user privileges
- `Elevated`: Running with elevated/administrator privileges
- `Restricted`: Running with restricted privileges

## Related functions

- [`if()`][00] - Conditional value selection
- [`equals()`][01] - Compare values for equality
- [`not()`][02] - Logical negation
- [`concat()`][03] - Concatenate strings together

<!-- Link reference definitions -->
[00]: if.md
[01]: equals.md
[02]: not.md
[03]: concat.md
