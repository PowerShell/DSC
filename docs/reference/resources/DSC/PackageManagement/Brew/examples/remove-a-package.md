---
description: > 
    Demonstrates how to remove a package with the DSC.PackageManagement/Brew resource
ms.date:     07/03/2025
ms.topic:    reference
title:       Remove a package with Brew
---

# Remove a package with Brew

This example demonstrates how to use the `DSC.PackageManagement/Brew` resource to remove a package
on MacOS systems using Brew.

## Test if package is installed

The following snippet shows how you can use the resource with the [dsc resource test][00] command
to check whether the `node` package doesn't exists.

```bash
dsc resource test --resource DSC.PackageManagement/Brew --input '{"packageName":"node","_exist":false}'
```

When the package is installed, DSC returns the following result.

```yaml
desiredState:
  packageName: node
  _exist: false
actualState:
  _exist: true
  packageName: node
  version: "24.3.0"
inDesiredState: false
differingProperties:
  - _exist
```

## Ensure a package is removed

To ensure the system is in the desired state, use the [dsc resource set][01]
command.

```bash
dsc resource set --resource DSC.PackageManagement/Brew --input '{"packageName":"node","_exist":false}'
```

When the resource removes the package, DSC returns the following result:

```yaml
beforeState:
  packageName: "node"
  version: "24.3.0"
  _exist: true
afterState:
  _exist: false
  packageName: node
  version: ""
changedProperties:
- _exist
- version
```

You can test the instance again to confirm that the package has been removed:

```bash
dsc resource test --resource DSC.PackageManagement/Brew --input '{"packageName":"node","_exist":false}'
```

```yaml
desiredState:
  packageName: node
  _exist: false
actualState:
  _exist: false
  packageName: node
  version: ""
inDesiredState: true
differingProperties: []
```

<!-- Link reference definitions -->
[00]: ../../../../../cli/resource/test.md
[01]: ../../../../../cli/resource/set.md
