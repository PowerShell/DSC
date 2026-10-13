---
description: Microsoft.PowerShell/Discover extension reference documentation
ms.date:     08/08/2026
ms.topic:    reference
title:       Microsoft.PowerShell/Discover
---

# Microsoft.PowerShell/Discover

## Synopsis

Discovers DSC resources packaged in PowerShell 7 modules.

## Metadata

```yaml
Version      : 0.1.1
Kind         : extension
Capabilities : [discover]
Author       : Microsoft
```

## Condition

The extension only applies on systems where the `pwsh` executable is available in `PATH`. DSC skips
the extension if `pwsh` isn't found.

DSC uses the following expression to evaluate whether to load and use this extension:

```yaml
condition: "[not(equals(tryWhich('pwsh'), null()))]"
```

## Description

By default, DSC discovers command resources by searching the folders in the `PATH` or
[`DSC_RESOURCE_PATH`][01] environment variable for manifest files. The
`Microsoft.PowerShell/Discover` extension expands that discovery to PowerShell 7 modules.

This extension recursively searches every folder in the `PSModulePath` environment variable for DSC
manifest files and returns their paths to DSC.

With this extension, publishers can package command resources inside a PowerShell module and
distribute the module through the PowerShell Gallery or a private repository. After a user
installs the module, for example with `Install-PSResource`, DSC discovers the packaged resources
automatically - the module folder doesn't need to be added to `PATH` or `DSC_RESOURCE_PATH`.

The extension searches for files with the following extensions, in any folder of the module:

- `.dsc.resource.json`, `.dsc.resource.yaml`, `.dsc.resource.yml`
- `.dsc.adaptedresource.json`, `.dsc.adaptedresource.yaml`, `.dsc.adaptedresource.yml`
- `.dsc.manifests.json`, `.dsc.manifests.yaml`, `.dsc.manifests.yml`
- `.dsc.extension.json`, `.dsc.extension.yaml`, `.dsc.extension.yml`

> [!NOTE]
> This extension discovers DSC _manifests_ that happen to ship inside a PowerShell module. It
> doesn't make class-based PowerShell DSC (PSDSC) resources available to DSC. The
> [Microsoft.Adapter/PowerShell][02] adapter discovers and invokes PSDSC resources.
>
> Folders for Windows PowerShell modules in `PSModulePath` are excluded from the search.

## Caching

Searching every module folder recursively can be slow on systems with many modules, so the
extension caches its results between invocations. The cache is stored at the following location:

- On Windows: `%LOCALAPPDATA%\dsc\PowerShellDiscoverCache.json`
- On Linux and macOS: `~/.dsc/PowerShellDiscoverCache.json`

The extension rescans the module folders and rebuilds the cache when any of the following change:

- The set of folders in `PSModulePath` differs from the cached set.
- The last-write time of a module folder, or of a module subfolder, differs from the cached time.
- A cached manifest file no longer exists on disk.

To force a full rescan, delete the cache file.

## Requirements

- PowerShell 7 (`pwsh`) must be available in `PATH`.

## Examples

List the extension and verify it's available:

```sh
dsc extension list Microsoft.PowerShell/Discover
```

```Output
Type                           Version  Capabilities  Description
--------------------------------------------------------------------------------------------------------------
Microsoft.PowerShell/Discover  0.1.1    d--           Discovers DSC resources packaged in PowerShell 7 modules.
```

When the extension is available, resources packaged in installed PowerShell modules appear in the
output of [dsc resource list][03] alongside resources discovered through `PATH`, and you can use
them in configuration documents and resource commands like any other resource.

## See also

- [Microsoft.Windows.Appx/Discover extension][04]
- [dsc extension list][05]
- [DSC extension manifest discover property schema reference][06]
- [DSC extension discover operation stdout schema reference][07]

<!-- Link reference definitions -->
[01]: ../../../../cli/index.md#dsc_resource_path
[02]: ../../../../resources/Microsoft/Adapter/PowerShell/index.md
[03]: ../../../../cli/resource/list.md
[04]: ../../Windows/Appx/Discover/index.md
[05]: ../../../../cli/extension/list.md
[06]: ../../../../schemas/extension/manifest/discover.md
[07]: ../../../../schemas/extension/stdout/discover.md
