---
description: Microsoft.Windows/WMI resource adapter reference documentation
ms.date:     03/25/2025
ms.topic:    reference
title:       Microsoft.Windows/WMI
---

# Microsoft.Windows/WMI

## Synopsis

Adapter for querying and retrieving information from Windows Management Instrumentation (WMI).

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [windows, wmi]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instanceName>
    type: Microsoft.Windows/WMI
    properties:
      # Required properties
      resources:
      - name: <nested instance name>
        type: <namespace name>/<class name>
        properties: # adapted resource properties

# Or from v3.1.0-preview.2 onwards
resources:
- name: <instanceName>
  type: <namespace name>/<class name>
  properties: # adapted resource properties

```

## Description

The `Microsoft.Windows/WMI` resource adapter enables you to query and retrieve information
from Windows Management Instrumentation (WMI). The resource can:

- Execute WMI queries to retrieve system information
- Filter WMI query results based on specific conditions
- Access data from different WMI namespaces

The adapter leverages PowerShell commands to retrieve and list information of WMI classes.

## Requirements

- The resource is only usable on a Windows system.
- The resource must run in a process context that has appropriate permissions to access WMI.

## Capabilities

The resource adapter has the following capabilities:

- `get` - You can use the resource to retrieve information from WMI.
- `list` - Lists available WMI classes that can be queried.

## Examples

1. [Query Operating System Information][01] - Shows how to query basic operating system information
2. [Query Filtered Disk Information][02] - Shows how to query disk drives with filtering

## Properties

## Property schema

WMI properties aren't exposed directly to a schema. To discover the available properties for a WMI class that you
can use in your configuration, run the following PowerShell command:

```powershell
dsc resource list --adapter Microsoft.Windows/WMI <namespace name>/<class name> |
  ConvertFrom-Json | 
  Select-Object properties
```

When defining a configuration document, the following properties are required.

### resources

The `resources` property defines a list of adapted WMI class instances that the adapter manages.
Every instance in the list must be unique, but instances may share the same DSC resource type.

For more information about defining a valid adapted resource instance, see the
[Adapted resource instances](#adapted-resource-instances) section of this document.

```yaml
Type:             array
Required:         true
MinimumItemCount: 1
ValidItemSchema:  https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.resource.json
```

## Adapted resource instances

Adapted resources instances always adhere to the
[DSC Configuration document resource instance schema](../../../../schemas/config/resource.md).

Every adapted instance must be an object that defines the [name](#adapted-instance-name),
[type](#adapted-instance-type), and [properties](#adapted-instance-properties) for the instance.

### Adapted instance name

The `name` property of the adapted resource instance defines the short, human-readable name for the
instance. The adapted instance name must be a non-empty string containing only letters, numbers,
and spaces. This property should be unique within the adapter's `resources` array.

> ![NOTE]
> The adapter doesn't currently raise an error when you define two adapted instances with the same
> name. In a future release, the adapter will be updated to emit a warning when adapted instances
> share the same name. In the next major version of the adapter, name conflicts will raise an
> error.
>
> Using the same name for multiple instances can make debugging and reviewing output more
> difficult. Always use unique names for every instance.

```yaml
PropertyName:  name
Type:          string
Required:      true
MinimumLength: 1
Pattern:       ^[a-zA-Z0-9 ]+$
```

### Adapted instance type

The `type` property identifies the adapted instance's WMI class resource. The value for this property
must be the valid fully qualified type name for the resource.

This adapter uses the following syntax for determining the fully qualified type name of a WMI class:

```Syntax
<namespace name>/<class name>
```

For example, if a WMI class named `Win32_OperatingSystem`, the fully qualified type name for that
resource is `root.cimv2/Win32_OperatingSystem`.

For more information about type names in DSC, see
[DSC Resource fully qualified type name schema reference][03].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### Adapted instance properties

The `properties` of an adapted resource instance define its desired state. The value of this
property must be an object. In case of the WMI adapter resource, properties are added at runtime
when the adapter tries to execute.

Each name for each property returns the filtered state. The property name isn't case  sensitive.

[!NOTE]
> The current WMI adapter doesn't warn or raise an error when an invalid property is passed.


```yaml
Type:     object
Required: true
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because the WMI query could not be executed successfully.
When the resource returns this exit code, it also emits an error message with details about the failure.

<!-- Link definitions -->
[01]: ./examples/query-operating-system-info.md
[02]: ./examples/query-filtered-disk-info.md
[03]: ../../../../schemas/config/type.md