---
description: Microsoft.OpenSSH.SSHD/sshd_config resource reference documentation
ms.date: 07/15/2025
ms.topic: reference
title: Microsoft.OpenSSH.SSHD/sshd_config
---

# Microsoft.OpenSSH.SSHD/sshd_config

## Synopsis

Manage SSH Server Configuration.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [OpenSSH, Windows, Linux]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.OpenSSH.SSHD/sshd_config
    properties:
    # Required properties
      map: object
```

## Description

The `Microsoft.OpenSSH.SSHD/sshd_config` resource allows you to export client
and server configuration settings. The resource can:

- Export client and server configuration settings

> [!NOTE]
> This resource is installed with DSC itself on systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource requires OpenSSH server and client to be installed on the Windows system.
- The resource must run at least under a Windows Server 2019 or Windows 10 (build 1809)
  operating system.

## Capabilities

The resource has the following capabilities:

- `export` - You can use the resource to export the current SSH server configuration.

## Examples

1. [Export OpenSSH configuration][00] - Shows how to export current OpenSSH configuration.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource. An instance that doesn't define each of these
  properties is invalid. For more information, see the "Required resource properties" section in
  [DSC resource properties][01]

  - [map](#map) - 

- **Key properties:** <a id="key-properties"> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][02].

  - [map](#map) (required) - 

### map

<details><summary>Expand for <code>map</code> property metadata</summary>

```yaml
Type             : object
IsRequired       : true
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource. The
validating schema only includes schema keywords that affect how the instance is validated. All
non validating keywords are omitted.

```json
{
"type": "object",
  "required": [
    "map"
  ],
  "properties": {
    "map": {
      "type": "object",
      "additionalProperties": true
    }
  }
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Invalid parameter

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed due to an invalid parameter. When the resource returns this
exit code, it also emits an error message with details about the invalid parameter.

## See also

- [Microsoft.OpenSSH.SSHD/Windows resource][03]
- For more information about OpenSSH, see [OpenSSH Documentation][04]

<!-- Link definitions -->
[00]: examples/export-openssh-configuration.md
[01]: ../../../../../concepts/resources/properties.md#required-resource-properties
[02]: ../../../../../concepts/resources/properties.md#key-resource-properties
[03]: ../Windows/index.md
[04]: /windowsserverdocs/WindowsServerDocs/administration/OpenSSH/openssh-overview
