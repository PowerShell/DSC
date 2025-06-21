---
description: >-
  Learn about Microsoft's Desired State Configuration platform, including what it does and when
  it should be used.
ms.date: 03/25/2025
ms.topic: overview
title:  Microsoft Desired State Configuration overview
---

# Microsoft Desired State Configuration overview

Microsoft's Desired State Configuration (DSC) is a declarative configuration platform. With DSC,
the state of a machine is described using a format that should be clear to understand even if the
reader isn't a subject matter expert. Unlike imperative tools, with DSC the definition of an
application environment is separate from programming logic that enforces that definition.

The DSC command line application (`dsc`) abstracts the management of software components
declaratively and idempotently. DSC runs on Linux, macOS, and Windows without any external
dependencies.

With DSC, you can:

- Author DSC Resources to manage your systems in any language.
- Invoke individual resources directly.
- Create configuration documents that define the desired state of a system.

## Configuration Documents

DSC Configuration Documents are declarative data files that define instances of resources.
Typically, configuration documents define what state to enforce. DSC supports writing configuration
documents in both JSON and YAML.

Example scenarios include requirements for an application environment or operational/security
standards.

## DSC Resources

DSC Resources define how to manage state for a particular system or application component.
Resources describe a schema for the manageable settings of the component. Every resource can be
used with the **Get** and **Test** operations to retrieve the current state of a resource instance
and validate whether it's in the desired state. Most resources also support enforcing the desired
state with the **Set** operation.

Example scenarios include:

- How to update the contents of a file.
- How to run a utility that changes the state of a machine.
- How to configure settings of an application.

### Differences from PowerShell DSC

DSC differs from PowerShell Desired State Configuration (PSDSC) in a few important ways:

- DSC doesn't _depend_ on PowerShell, Windows PowerShell, or the [PSDesiredStateConfiguration][01]
  PowerShell module. DSC provides full compatibility with PSDSC resources through the
  `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WindowsPowerShell` _adapter resources_.

  With the `Microsoft.DSC/PowerShell` adapter resource, you can use any PSDSC resource implemented
  as a PowerShell class. The resource handles discovering, validating, and invoking PSDSC
  resources in PowerShell. The resource is included in the DSC install package for every platform.

  With the `Microsoft.Windows/WindowsPowerShell` adapter resource, you can use any PSDSC resource
  compatible with Windows PowerShell. The resource handles discovering, validating, and invoking
  PSDSC resources in Windows PowerShell. The resource is included in the DSC install packages for
  Windows only.
- Because DSC doesn't depend on PowerShell, you can use DSC without PowerShell installed and manage
  resources written in bash, Python, C#, Rust, or any other language.
- DSC doesn't include a local configuration manager. DSC is invoked as a command. It doesn't
  run as a service.
- New DSC resources define their schemas with JSON or YAML files, not MOF files. Self-contained
  resources define a _resource manifest_ that indicates how DSC should invoke the resource and what
  properties the resource can manage. For adapted resources, like those implemented in PowerShell,
  the adapter resource tells DSC what the available properties are for the resource and handles
  invoking the adapted resources.
- Configuration documents are defined in JSON or YAML files, not PowerShell script files.
  Configuration documents support a subset of functionality in ARM templates, including parameters,
  variables, metadata, and expression functions to dynamically resolve data in the configuration.

## Installation

### Install DSC manually

To install DSC on any platform:

1. Download the [latest release from the PowerShell/DSC repository][02].
1. Expand the release archive.
1. Add the folder containing the expanded archive contents to the `PATH`.

> [!NOTE]
> When downloading the latest release on Windows platform, make sure after extraction,
> the files are unblocked. You can do this using the following PowerShell command:
>
> ```powershell
> Get-ChildItem -Path <path-to-expanded-folder> -Recurse | Unblock-File
> ```

### Install DSC on Windows with WinGet

The following commands can be used to install DSC using the published `winget` packages:

Search for the latest version of DSC

```powershell
winget search DesiredStateConfiguration
```

```Output
Name                              Id           Version Source
---------------------------------------------------------------
DesiredStateConfiguration         9NVTPZWRC6KQ Unknown msstore
DesiredStateConfiguration-Preview 9PCX3HX4HZ0Z Unknown msstore
```

Install DSC using the `id` parameter:

```powershell
# Install latest stable
winget install --id 9NVTPZWRC6KQ --source msstore
```

```powershell
# Install latest preview
winget install --id 9PCX3HX4HZ0Z --source msstore
```

## Integrating with DSC

DSC is a platform tool that abstracts the concerns for defining and invoking resources. Higher
order tools, like [WinGet][03], [Microsoft Dev Box][04], and [Azure Machine Configuration][05] are
early partners for DSC as orchestration agents.

DSC uses JSON schemas to define the structure of resources, configuration documents, and the
outputs that DSC returns. These schemas make it easier to integrate DSC with other tools, because
they standardize and document how to interface with DSC.

For more information, see [DSC JSON Schema reference overview][06].

## See Also

- [Anatomy of a command-based DSC Resource][07] to learn about authoring a resource in your
  language of choice.
- [Command line reference for the 'dsc' command][08]
- [DSC JSON Schema reference overview][06]
- [WinGet Configuration][09]

<!-- link references -->
[01]: https://github.com/powershell/psdesiredstateconfiguration
[02]: https://github.com/PowerShell/DSC/releases/latest
[03]: /windows/package-manager/winget
[04]: /azure/dev-box/overview-what-is-microsoft-dev-box
[05]: /azure/governance/machine-configuration/overview
[06]: ./reference/schemas/overview.md
[07]: ./concepts/resources/anatomy.md
[08]: ./reference/cli/index.md
[09]: /windows/package-manager/configuration/
