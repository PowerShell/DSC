---
RFC:          RFCNNNN # WG will set the number after submission
Author:       "@SteveL-MSFT"
Sponsor:      null    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      1.0     # <Major>.<Minor>
Area:         Extension
CommentsDue:  null    # <Date for submitting comments to current draft (minimum 1 month)>
---

# DSC Registry

This spec will use the term artifacts to refer to resources, extensions, plugins, and configurations.

To make it easier for users to discover and install artifacts there should be an extension to DSC to support this.

The current `discover` extension is limited to resources found on the current system.
What needs to be supported is remote discovery and runtime install of artifacts.

## Motivation

> As an ITPro,
> I want author DSC configuration without installing all artifacts onto my development system,
> so that I can then deploy the configuration at-scale.

## Proposed experience

### Configuration Authoring Experience

When authoring a configuration, `dsc resource list` should find and list any resources matching
the provided filters locally and from remote registry(s).

Similarly, `dsc extension list` would find and list any extensions locally and from remote registry(s).

For higher order tools, such as a VSCode Extension, the `list_dsc_resources()` JSONRPC API should
work the same way.
This would allow the VSCode Extension to provide intellisense for resources the user trusts but
is not currently installed on their system.

The returned resources match the current local experience including the schema embedded within
the resource manifest.

### Configuration Deployment Experience

A configuration that is invoked will find required resources using the remote registry(s) if not
found locally, install them to a local cache, and use them during deployment.

Add a new policy (aligned with structured policy/settings design) that restricts resources used during deployment to only those from trusted registries.
Core resources shipped with DSC will be in a built-in allow list.

### Non-Goals

The publishing of resources and configurations are not part of this RFC.
Integration of a publishing experience with `dsc`, if needed, can be covered later.

## Specification

### Discovery

The `discover` extension schema will add a new `remote` property object with a `url` property.
This url points to how to download the resource.
If `url` is used by the extension, then it MUST use `manifestContent` and embed the manifest.
The `type` within the manifest is used to associate to a registered registry (see below).

Example manifest:

```yaml
type: <string>
discover:
  executable: <string>
  args: [array of args]
remoteDiscover:
  executable: <string>
  args: [array of args]
```

> [!NOTE]
> Extensions can implement both or just one of `discover` or `remoteDiscover`

### Installation

A new `install` extension will be added to be used in conjunction with a `discover` extension.
Although the same executable can be used for both, there will need to be separate manifests
(although they can be in a single manifests file).

The extension manifest will resemble:

```yaml
type: <string>
install:
  executable: <string>
  args: [array of args]
```

One of the args will be `cacheArg` which provides the disk location to save the resource or configuration.

Resources in the cache directory will use the required convention `resource/<packageName>/<resourceType>/<version>` as subfolders.

Since a single package can contain multiple resources, the package name just needs to be unique.

Extensions will use the required convention `extension/<extensionType>/<version>` as subfolders.
Configurations in the cache will be stored under the `configuration` folder.

> Open Questions:
>
> - Resource types have `<namespace>/<type>` format which wouldn't work as a folder.
>   One solution is to have the namespace be a folder and `<type>` as subfolder.

### Registry

A new `registry` extension model will allow interacting with registries on the system.
Registered registries are inherently trusted.

> Open Questions:
>
> - Is there a scenario where we want to support untrusted registries?

The extension manifest will resemble:

```yaml
type: <string>
registry:
  executable: <string>
  args: [array of args]
```

> Open Questions:
>
> How to support scope and policy vs preference?

`dsc registry list` will show all registered registries.

`dsc registry add --URL <url> --type <extensionType> --priority <num>` will register a new registry.  The `<url>` scheme identifies which extension to associate to the URL.

`dsc registry remove <name>` will unregister a registry by name.

`registry` extension's role is to associate a registry to a `discover` and `install` extension via the scheme.

### Registry Authentication

It will not be possible to abstract all the different ways the user can authenticate with a specific registry.
Expectation is that registry extensions will expose their own DSC resource for managing those settings.

### Caching

A local cache of the resources will be created for the user under:

- Windows
  - `$env:APPDATA\dsc\cache`
- Linux and macOS
  - `$HOME/.config/dsc/cache`

There will be settings added to the registry registration on how the cache is cleaned up:

- Diskspace size
- Recently used last n days (default, 30 days)
- After every deployment

The DSC engine will be responsible for cleaning the cache and extensions do not play a role.

> Open Questions:
>
> - Generalize caching as an API for adapters/extensions to use/participate

### Default extensions

The `discover`, `install`, and `registry` extensions shipped with DSC will support [OCI](https://opencontainers.org/)
via [ORAS](https://oras.land/) which are projects that are part of the [Cloud Native Computing Foundation](https://www.cncf.io/).

Details of these extensions will be a separate RFC.

The extension model allows for 3rd party extensions that can support other transports and registry types.

## Alternate Proposals and Considerations

<!--
    Include any alternate proposals and notes for the RFC in this section.
-->

## Related work items

- [#92](https://github.com/PowerShell/DSC/issues/92) - DSC Registry Proposal
- [#826](https://github.com/PowerShell/DSC/issues/826) - Runtime download of necessary resources from trusted registries
- [#1763](https://github.com/PowerShell/DSC/issues/1763) - Separate our resources as separate package
- [#461](https://github.com/PowerShell/DSC/issues/461) - Add ability to have plugins to dsc.exe
