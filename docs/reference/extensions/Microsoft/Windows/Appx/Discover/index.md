---
description: Microsoft.Windows.Appx/Discover extension reference documentation
ms.date:     08/08/2026
ms.topic:    reference
title:       Microsoft.Windows.Appx/Discover
---

# Microsoft.Windows.Appx/Discover

## Synopsis

Discovers DSC resources packaged as Appx packages.

## Metadata

```yaml
Version      : 0.1.0
Kind         : extension
Capabilities : [discover]
Author       : Microsoft
```

## Description

By default, DSC discovers command resources by searching the folders in the `PATH` or
[`DSC_RESOURCE_PATH`][01] environment variable for manifest files. The
`Microsoft.Windows.Appx/Discover` extension expands that discovery to Appx packages on Windows:
it enumerates the Appx packages installed for the current user and searches the installation
folder of each package for DSC manifest files, returning their paths to DSC.

With this extension, publishers can include a DSC resource in an application that's distributed
through the Microsoft Store or installed as an MSIX/Appx package. After a user installs the
application, DSC discovers the packaged resources automatically - the application's installation
folder doesn't need to be added to `PATH` or `DSC_RESOURCE_PATH`.

The extension searches the root of each package's installation folder for files with the
following extensions:

- `.dsc.resource.json`, `.dsc.resource.yaml`, `.dsc.resource.yml`
- `.dsc.adaptedresource.json`, `.dsc.adaptedresource.yaml`, `.dsc.adaptedresource.yml`
- `.dsc.manifests.json`, `.dsc.manifests.yaml`, `.dsc.manifests.yml`
- `.dsc.extension.json`, `.dsc.extension.yaml`, `.dsc.extension.yml`

> [!NOTE]
> The extension searches only the root of each package's installation folder, not its
> subfolders. Manifests must be placed at the top level of the package.

## Requirements

- The extension only applies on Windows. It uses Windows PowerShell and the `Get-AppxPackage`
  cmdlet to enumerate installed packages.
- The extension enumerates the Appx packages installed for the current user.

## Examples

List the extension and verify it's available:

```sh
dsc extension list Microsoft.Windows.Appx/Discover
```

```Output
Type                             Version  Capabilities  Description
------------------------------------------------------------------------------------------------------
Microsoft.Windows.Appx/Discover  0.1.0    d--           Discovers DSC resources packaged as Appx packages.
```

When the extension is available, resources packaged in installed Appx packages appear in the
output of [dsc resource list][02] alongside resources discovered through `PATH`, and you can use
them in configuration documents and resource commands like any other resource.

## See also

- [Microsoft.PowerShell/Discover extension][03]
- [dsc extension list][04]
- [DSC extension manifest discover property schema reference][05]
- [DSC extension discover operation stdout schema reference][06]

<!-- Link reference definitions -->
[01]: ../../../../../cli/index.md#dsc_resource_path
[02]: ../../../../../cli/resource/list.md
[03]: ../../../PowerShell/Discover/index.md
[04]: ../../../../../cli/extension/list.md
[05]: ../../../../../schemas/extension/manifest/discover.md
[06]: ../../../../../schemas/extension/stdout/discover.md
