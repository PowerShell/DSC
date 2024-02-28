---
description: Reference for the 'resourceId' DSC configuration document function
ms.date:     01/17/2024
ms.topic:    reference
title:       resourceId
---

# resourceId

## Synopsis

Returns the unique identifier of a resource.

## Syntax

```Syntax
resourceId('<resourceTypeName>', '<instanceName>')
```

## Description

The `resourceId()` function returns a handle to a specific resource instance in the configuration.
This function enables instances to reference another instance for the [dependsOn][01] option.

> [!NOTE]
> When using the `resourceId` function for [nested resource instances][02], instances can only
> reference other instances in the same resource adapter or group instance. They can't use the
> `resourceId()` function to lookup instances at the top-level of the configuration document or
> inside another adapter or group instance.

## Examples

### Example 1 - Reference a resource as a dependency

The following configuration uses the `resourceId()` function to reference the instance named
`Tailspin Key` as a dependency of the `Update Tailspin Automatically` resource instance.

```yaml
# resourceId.example.1.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
  - name: Tailspin Key
    type: Microsoft.Windows/Registry
    properties:
      keyPath: HKCU\tailspin
      _ensure: Present
  - name: Update Tailspin Automatically
    type: Microsoft.Windows/Registry
    properties:
      keyPath:   HKCU\tailspin\updates
      valueName: automatic
      valueData:
        String: enable
    dependsOn:
      - "[resourceId('Microsoft.Windows/Registry', 'Tailspin Key')]"
```

### Example 2 - Reference a group resource as a dependency

The following configuration uses the `resourceId()` function to specify the `DSC/AssertionGroup`
resource instance named 'IsWindows' as a dependency of the `Example Key` resource instance.

```yaml
# resourceId.example.2.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
  - name: IsWindows
    type: DSC/AssertionGroup
    properties:
      $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
      resources:
        - name: os
          type: Microsoft/OSInfo
          properties:
            family: Windows
  - name: Example Key
    type: Microsoft.Windows/Registry
    properties:
      keyPath: HKCU\example
      _exist: true
```

## Parameters

### resourceTypeName

The value of the [type][03] property of the resource instance to reference. The value must be the
[fully qualified type name][04] for the resource.

```yaml
Type:     string
Required: true
Position: 0
```

### instanceName

The value of the [name][05] property of the resource instance to reference.

```yaml
Type:     string
Required: true
Position: 0
```

<!-- Link reference definitions -->
[01]: ../resource.md#dependson
[02]: /powershell/dsc/glossary#nested-resource-instance
[03]: ../resource.md#type
[04]: ../../definitions/resourceType.md
[05]: ../resource.md#name
