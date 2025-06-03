---
description: Microsoft.Windows/RebootPending resource reference documentation
ms.date:     03/25/2025
ms.topic:    reference
title:       Microsoft.Windows/RebootPending
---

# Microsoft.Windows/RebootPending

## Synopsis

Checks if a Windows system has a pending reboot.

> [!IMPORTANT]
> The `Microsoft.Windows/RebootPending` resource are a proof-of-concept example
> for use with DSC. Don't use it in production.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/RebootPending
    properties: {}
```

## Description

The `Microsoft.Windows/RebootPending` resource enables you to check whether a Windows system has a pending reboot. The resource can determine if a system reboot is required due to:

- Windows Updates
- Component-Based Servicing
- Pending file rename operations
- Pending computer rename
- Pending domain join operations

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- The resource must run in a process context that has permissions to query the system for reboot status.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the pending reboot status of a system.

This resource does not support `set`, `whatIf`, `export`, `test`, or `delete` operations. For more information about resource capabilities, see [DSC resource capabilities][02].

## Examples

1. [Check for pending reboot][03] - Shows how to check if a system has a pending reboot using the `dsc resource get` command.
2. [Use the RebootPending resource in a configuration][04] - Shows how to include the RebootPending resource in a configuration document to check reboot status.

## Properties

The resource doesn't have any configurable properties. It's a read-only resource designed to detect a system's reboot status.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following properties. For more information about read-only properties, see the "Read-only resource properties" section in [DSC resource properties][08].

  - [rebootPending](#rebootpending) - Indicates whether the system has a pending reboot.  

### rebootPending

<details><summary>Expand for <code>rebootPending</code> property metadata</summary>

```yaml
Type         : boolean
IsRequired   : false
IsKey        : false
IsReadOnly   : true
IsWriteOnly  : false
```

</details>

A boolean value that indicates whether the system has a pending reboot. `true` if a reboot is pending; otherwise, `false`.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
"type": "null",
  "properties": {
    "rebootPending": {
      "type": "boolean",
      "readOnly": true
    }
  }
}
```

## Exit codes

The resource doesn't return any specific exit codes. It reports status through the `rebootPending` property.

## See also

- [Microsoft.Windows/Registry resource][10]
- [Windows Registry][11]
- [DSC resource capabilities][02]
- [DSC resource properties][06]

<!-- Link definitions -->
[02]: ../../../../../concepts/resources/capabilities.md
[03]: ./examples/check-for-pending-reboot.md
[04]: ./examples/use-rebootpending-in-configuration.md
[06]: ../../../../../concepts/resources/properties.md
[08]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[10]: ../registry/index.md
[11]: /windows/win32/sysinfo/about-the-registry
