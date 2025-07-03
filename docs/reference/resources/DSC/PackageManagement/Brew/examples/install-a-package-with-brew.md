---
description: > 
    Demonstrates how to install a package with the DSC.PackageManagement/Brew resource
ms.date:     07/03/2025
ms.topic:    reference
title:       Install a package with Brew
---

# Install a package with Brew

This example demonstrates how to use the `DSC.PackageManagement/Brew` resource to install a package
on MacOS systems using Brew.

## Test if package is installed

The following snippet shows how you can use the resource with the [dsc resource test][00] command
to check whether the `node` package exists.

```bash
dsc resource test --resource DSC.PackageManagement/Brew --input '{"packageName":"node"}'
```

When the package is not installed, DSC returns the following result.

```yaml
desiredState:
  packageName: node
actualState:
  _exist: false
  packageName: node
  version: ""
inDesiredState: false
differingProperties:
  - _exist
```

## Ensure a package is installed

To ensure the system is in the desired state, use the [dsc resource set][01]
command.

```bash
dsc resource set --resource DSC.PackageManagement/Brew --input '{"packageName":"node"}'
```

When the resource installs the package, DSC returns the following result:

```yaml
beforeState:
  packageName: "node"
  version: ""
  _exist: false
afterState:
  _exist: true
  packageName: node
  version: "24.3.0"
changedProperties:
- _exist
- version
```

> [!NOTE]
> Note that the version can differ depending on your system's package repositories
> and available package versions.

You can test the instance again to confirm that the package exists:

```bash
dsc resource test --resource DSC.PackageManagement/Brew --input '{"packageName":"node"}'
```

```yaml
desiredState:
  packageName: node
actualState:
  _exist: true
  packageName: node
  version: "24.3.0"
inDesiredState: true
differingProperties: []
```

<!-- Link reference definitions -->
[00]: ../../../../../cli/resource/test.md
[01]: ../../../../../cli/resource/set.md
