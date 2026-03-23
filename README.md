# DSCv3

> [!NOTE]
> We welcome code contributions to this repository. For guidelines on how to contribute,
> see our [CONTRIBUTING.md](CONTRIBUTING.md).
> Your feedback and participation help us improve DSCv3 for everyone.

## What's DSCv3?

DSCv3 is the latest iteration of Microsoft's Desired State Configuration platform. DSCv3 is an open
source command line application that abstracts the management of software components declaratively
and idempotently. DSCv3 runs on Linux, macOS, and Windows without any external dependencies.

With DSCv3, you can:

- Author resources to manage your systems in any language
- Invoke individual resources
- Create configuration documents that define the desired state of a system

### Differences from PowerShell DSC

DSCv3 differs from PowerShell DSC in a few important ways:

- DSCv3 doesn't depend on PowerShell. You can use DSCv3 without PowerShell installed and manage
  resources written in bash, python, C#, Go, or any other language.
- DSCv3 use of PowerShell based resources does not depend on [PSDesiredStateConfiguration][00] module
- DSCv3 doesn't include a local configuration manager. DSCv3 is invoked as a command. It doesn't
  run as a service.
- Non-PowerShell resources define their schemas with JSON files, not MOF files.
- Configuration documents are defined in JSON or YAML files, not PowerShell script files.

Importantly, while DSCv3 represents a major change to the DSC platform, DSCv3 is able to invoke
PowerShell DSC Resources, including script-based and class-based DSC Resources, as they exist today. The
configuration documents aren't compatible, but all published PowerShell DSC Resources are. You can
use PowerShell DSC resources in DSCv3 with both Windows PowerShell and PowerShell.

## Installing DSCv3

To install DSC v3:

1. Download the [latest release from this repository][01].
1. Expand the release archive.
1. Add the folder containing the expanded archive contents to the `PATH`.

For information on building and testing DSC from source, see the [Contributing Guide](CONTRIBUTING.md).

## Explore DSCv3

If you're new to DSC or configuration management, we recommend reviewing the [documentation][02].

### Learning and authoring tutorials

If you're already familiar with DSC or just want to start experimenting with v3, we recommend
reviewing the [Samples repository][03] and the accompanying [tutorial site][04]. The Samples
repo is ready to accept Pull Requests, which is a great way to contribute while v3 is in early
phases of development.

## Integrating with DSCv3

DSCv3 is a platform tool that abstracts the concerns for defining and invoking resources. Higher
order tools, like Azure Machine Configuration, Azure Automanaged VM, and WinGet are early partners
for DSCv3 as orchestration agents.

DSCv3 uses JSON schemas to define the structure of resources, configuration documents, and the
outputs that DSCv3 returns. These schemas make it easier to integrate DSCv3 with other tools,
because they standardize and document how to interface with DSCv3.

## Code of Conduct

Please see our [Code of Conduct](CODE_OF_CONDUCT.md) before participating in this project.

## Security Policy

For any security issues, please see our [Security Policy](SECURITY.md).

[00]: https://github.com/powershell/psdesiredstateconfiguration
[01]: https://github.com/PowerShell/DSC/releases/latest
[02]: https://learn.microsoft.com/powershell/dsc/overview?view=dsc-3.0&preserve-view=true
[03]: https://github.com/PowerShell/DSC-Samples
[04]: https://powershell.github.io/DSC-Samples
