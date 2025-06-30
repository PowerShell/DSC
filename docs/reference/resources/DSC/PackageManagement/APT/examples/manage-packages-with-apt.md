---
description: > 
    Demonstrates how to manage packages with the DSC.PackageManagement/Apt resource
ms.date:     06/30/2025
ms.topic:    reference
title:       Manage packages with APT
---

# Manage packages with APT

This example demonstrates how to use the `DSC.PackageManagement/Apt` resource to manage packages on Linux systems
that use the APT package manager.

## Test if package is installed

The following snippet shows how you can use the resource with the [dsc resource test][01] command
to check whether the `nginx` package exists.

```bash
dsc resource test --resource DSC.PackageManagement/Apt --input '{"packageName":"nginx"}'
```

When the package is not installed, DSC returns the following result.

> [!NOTE]
> Note that the version and source values can differ depending on your system's package repositories and available
> package versions.

```yaml
desiredState:
  packageName: nginx
actualState:
  _exist: "true"
  packageName: nginx
  version: 1.24.0-2ubuntu7.3
  source: noble-updates,noble-security,now
inDesiredState: false
differingProperties:
  - _exist
```

## Ensure a package with version is installed

To ensure the system is in the desired state with a particular version, use the [dsc resource set][02]
command.

```bash
dsc resource set --resource DSC.PackageManagement/Apt --input '{"packageName":"nginx", "version":"1.24.0-2ubuntu7"}'
```

When the resource install the package, DSC returns the following result:

```yaml
beforeState:
  packageName: "nginx"
  version: "1.24.0-2ubuntu7"
  _exist: false
afterState:
  keyPath: HKCU\DscExamples\ManagedKey
  version: "1.24.0-2ubuntu7"
changedProperties:
- _exist
```

You can test the instance again to confirm that the package exists:

```bash
dsc resource test --resource DSC.PackageManagement/Apt --input '{"packageName":"nginx", "version":"1.24.0-2ubuntu7"}'
```

```yaml
desiredState:
  packageName: nginx
  version: 1.24.0-2ubuntu7
actualState:
  _exist: true
  packageName: nginx
  version: 1.24.0-2ubuntu7
  source: noble-updates,noble-security,now
inDesiredState: true
differingProperties: []
```

## Uninstall a package

To uninstall a package, set the `_exist` property to `false`:

```bash
dsc resource set --resource DSC.PackageManagement/Apt --input '{"packageName":"nginx", "_exist": false}'
```

The `DSC.PackageManagement/Apt` resource implements the [setHandleExist][03], indicating the resource
will be deleted.

To verify the package no longer exists, use the `dsc resource get` command

```powershell
dsc resource get --resource DSC.PackageManagement/Apt --input '{"packageName":"nginx"}'
```

```yaml
actualState:
  packageName: nginx
  _exist: false
```

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/test.md
[02]: ../../../../../cli/resource/set.md
[03]: ../../../../../../concepts/resources/capabilities.md#sethandlesexist
