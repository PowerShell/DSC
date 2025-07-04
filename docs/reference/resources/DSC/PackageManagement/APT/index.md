---
description: DSC.PackageManagement/Apt resource reference documentation
ms.date:     06/30/2025
ms.topic:    reference
title:       DSC.PackageManagement/Apt
---

# DSC.PackageManagement/Apt

## Synopsis

Manage packages with the advanced package tool (APT) on Linux systems.

> [!IMPORTANT]
> The `DSC.PackageManagement/Apt` resource is a proof-of-concept example
> for use with DSC. Don't use it in production.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Linux, apt, PackageManagement]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: DSC.PackageManagement/Apt
    properties:
      # Required properties
      packageName: string
      # Instance properties
      _exist: boolean
      version: string
```

## Description

The `DSC.PackageManagement/Apt` resource enables you to idempotently manage packages with the Advanced Package Tool (APT) on
Linux systems. The resource can:

- Install packages
- Uninstall packages
- Check if a package is installed
- Verify the version of an installed package

> [!NOTE]
> This resource only works on Linux systems that use the APT package manager, such as Ubuntu and Debian.

## Requirements

- A Linux system with APT installed
- Administrative privileges (root or sudo access) to install and remove packages

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `export` - You can use the resource to export the current state of the system.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][00].

## Examples

1. [Manage packages with APT](./examples/manage-packages-with-apt.md) - Shows how to install, uninstall,
   and check packages with the `DSC.PackageManagement/Apt` resource.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource. An instance that doesn't define each of these
  properties is invalid. For more information, see the "Required resource properties" section in
  [DSC resource properties][01]

  - [packageName](#packagename) - The name of the package to query or install.

- **Key properties:** <a id="key-properties"></a> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][02].

  - [packageName](#packagename) (required) - The name of the package to query or install.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [_exist](#_exist) - Defines whether the package should exist.
  - [version](#version) - The version of the package to install.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][03].

  - [source](#source) - Indicates the source of the package.

### packageName

<details><summary>Expand for <code>packageName</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : true
IsKey            : true
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the name of the package to query or install. This property is required and serves as the key for uniquely
identifying the package.

### _exist

<details><summary>Expand for <code>_exist</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
DefaultValue     : true
```

</details>

The `_exist` canonical resource property determines whether a package should exist. When the
value for `_exist` is `true`, the resource installs the package if it doesn't exist. When
the value for `_exist` is `false`, the resource removes or uninstalls the package if it does exist.
The default value for this property when not specified for an instance is `true`.

### version

<details><summary>Expand for <code>version</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the version of the package to install. If not specified, the latest available version will be installed.

### source

<details><summary>Expand for <code>source</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : true
IsWriteOnly      : false
```

</details>

Indicates the source of the package. This is a read-only property returned by the resource after a `get` operation.

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

- [For more information about APT](https://wiki.debian.org/Apt)

<!-- Link definitions -->
[00]: ../../../../../concepts/resources/capabilities.md
[01]: ../../../../../concepts/resources/properties.md#required-resource-properties
[02]: ../../../../../concepts/resources/properties.md#key-resource-properties
[03]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
