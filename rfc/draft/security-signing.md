---
RFC:          RFCNNNN # WG will set the number after submission
Author:       SteveL-MSFT
Sponsor:      null    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      1.0     # <Major>.<Minor>
Area:         Security
CommentsDue:  null    # <Date for submitting comments to current draft (minimum 1 month)>
---

# Trust and DSC Artifacts

To enable defense-in-depth security for DSC artifacts, this RFC proposes a signing mechanism for DSC artifacts. This will allow consumers of DSC artifacts to verify the authenticity and integrity of the artifacts they consume.
Artifacts include DSC configuration files, DSC manifests, and the executable used by the DSC manifest.
Only YAML format DSC artifacts will be supported for signing.

## Motivation

> As an Enterprise Administrator,
> I want to restrict the execution of DSC artifacts to only those that are signed by a trusted authority,
> so that I can ensure a secure supply chain for DSC artifacts.

## Proposed experience

Signing requirement will be opt-in, however, if it is not enabled, then a warning message will be emitted.
A policy setting will be added to enforce the signing requirement. If the policy is set to enforce signing, then unsigned artifacts will not be executed and an error message will be emitted.

The `DSC resource list` (and corresponding JSONRpc API) will be updated to include a `Trust` property for each resource.
The values will be `Authenticode`, `Catalog`, `Notary`, or `None`. This will allow consumers to determine the trust level of a resource before consuming it.

## Specification

DSC deployment at-scale is currently aligned with using OCI registries to store and distribute DSC artifacts.
Therefore, the signing mechanism will be aligned with the OCI registry signing mechanism.
The signing mechanism will be based on the [Notary Project](https://notaryproject.dev/) which specifically is for signing and verifying content in OCI registries.

There would be policy settings to set which signers are trusted, and which signing mechanism is required for a given artifact.

On Windows, there will be additional support for signing and verifying DSC artifacts using the Windows Authenticode signing mechanism.
This includes both Catalog signing (required for Windows in-box components such as Windows PowerShell) and individual File signing (binaries and YAML files).

Authenticode trust is determined by the Windows trust store, which is managed by the operating system.

Notary signing and authenticode signing are not mutually exclusive.

## Alternate Proposals and Considerations

PGP/GPG signing was considered, however, it was determined that the Notary Project is a better fit for the OCI registry signing mechanism.

On Linux/macOS, there will be resources that rely on binaries that are not part of the OCI artifact (e.g. python).
Since there isn't individual file signing on Linux/macOS, it may make sense to at least verify the folder of the binary has permissions that don't allow world write access.

## Related work items

- [Signing Resource Manifests](https://github.com/PowerShell/DSC/issues/327)
- [Signing Configurations](https://github.com/PowerShell/DSC/issues/210)
- [DSC Registry Proposal](https://github.com/PowerShell/DSC/issues/92)
