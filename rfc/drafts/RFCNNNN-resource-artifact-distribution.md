---
RFC:          RFCNNNN # WG will set the number after submission
Author:       "@SteveL-MSFT"
Sponsor:      null
Status:       Draft
SupercededBy: null
Version:      1.0
Area:         Resource distribution
CommentsDue:  2026-12-22
---

# Publishing, Discovering, and Caching DSC Resource Artifacts

This RFC defines a language-independent design for distributing DSC resources through an Open
Container Initiative (OCI) registry. Each resource has independently discoverable version and
platform metadata, while related resources can share an implementation ZIP. Consumers retrieve
individual resource manifests and expand implementations into a versioned, per-user cache.

The proposal specifies artifact identities, OCI documents, discovery and transfer behavior, and
cache consistency requirements. It does not define command-line interfaces, language bindings,
resource invocation, or integration with the DSC engine's local resource discovery.

## Motivation

> As a DSC resource consumer,
> I want to discover and acquire specific resource versions for my platform,
> so that I can use resources independently of the DSC release and retain multiple versions.

> As a DSC resource publisher,
> I want related resources to share their implementation package,
> so that I can publish each resource's metadata without duplicating identical implementation data.

A resource type is not necessarily an implementation package name. For example,
`Microsoft.Windows/Registry` and `Microsoft.Windows/RegistryList` share the Registry implementation.
Distribution must preserve that relationship without requiring consumers to know the package
mapping before searching.

Separating manifests from implementation ZIPs permits metadata-only discovery and independent
resource manifests. OCI provides content-addressed storage, platform indexes, and established
registry transport and authentication mechanisms.

## Proposed experience

A publisher prepares a package containing the Registry implementation and resource manifests.
Publication creates resource-specific indexes for Registry and RegistryList in the same package
repository. For a given platform, both resource artifacts reference identical ZIP bytes, but each
references its own DSC resource manifest.

A consumer searches using a full resource type and optional exact version and platform filters.
The following JSON illustrates a logical request, not a prescribed programming interface:

```json
{
  "resourceType": "Microsoft.Windows/RegistryList",
  "version": "1.0.0",
  "platform": "windows-amd64"
}
```

Search returns metadata identifying the resource, version, platform, package repository, resource
index reference, and digest-pinned platform artifact reference. Searching does not transfer DSC
resource manifest files or implementation ZIPs.

For acquisition, omitting the version selects the highest stable semantic version available for
the requested platform. Omitting the platform selects the host operating system and architecture.
An explicit version may select a prerelease.

The consumer downloads the selected resource manifest and implementation ZIP, verifies them, and
installs them together. On Windows, an example destination is:

```text
%LOCALAPPDATA%\dsc\resourceCache\Microsoft.Windows.Registry\1.0.0\
```

This directory contains `registry.exe`, `registrylist.dsc.resource.json`, and local cache metadata.
Acquiring Registry for the same package version and platform adds `registry.dsc.resource.json` and
reuses the implementation. Acquiring version `2.0.0` creates a sibling version directory rather
than replacing version `1.0.0`.

Successful acquisition reports the selected identity, digest-pinned reference, installation
directory, and resource manifest path. It does not execute the resource or alter resource search
paths.

## Specification

The terms MUST, SHOULD, and MAY express requirements of this proposal. The transport and document
envelopes follow the [OCI Distribution Specification][distribution], [OCI image manifest
specification][manifest], and [OCI image index specification][index].

### Resource and package identities

A resource identity consists of its full DSC type, resource version, and target platform. A type
contains a namespace and resource name separated by `/`, such as `Microsoft.Windows/RegistryList`.
The full type, not the resource name alone, is authoritative.

An implementation package is a named group of files shared by one or more resource types. Its
default name is the resource type with `/` replaced by `.`, such as `Microsoft.System.Process`.
Explicit grouping can assign several resources to one package, such as `Microsoft.Windows.Registry`.
The ZIP filename is `<package-name>.zip`.

Versions use [Semantic Versioning][semver]. The version recorded in the resource manifest and OCI
metadata MUST agree. In the cache layout, `<package-version>` is the selected resource artifact's
version, not the DSC engine version. This proposal does not introduce a separately versioned
package entity. Resources with different versions can reference identical ZIP bytes but occupy
different cache version directories.

Supported platforms are `linux-amd64`, `linux-arm64`, `darwin-amd64`, `darwin-arm64`,
`windows-amd64`, and `windows-arm64`. OCI descriptors represent these as separate `os` and
`architecture` strings. macOS maps to `darwin`, x64 maps to `amd64`, and ARM64 maps to `arm64`.
An unsupported host architecture MUST produce an explicit acquisition error rather than select
an arbitrary binary.

### Packaging inputs

Packaging metadata distinguishes implementation files (`zipContents`) from manifest sources
(`manifests`). For example:

```json
{
  "Microsoft.Windows.Registry": {
    "zipContents": ["registry.exe"],
    "manifests": [
      "resource-manifests/Microsoft.Windows.Registry.dsc.resource.json",
      "resource-manifests/Microsoft.Windows.RegistryList.dsc.resource.json"
    ]
  }
}
```

The repository's build metadata groups these definitions under `All`, `Linux`, `macOS`, and
`Windows`. A producer combines `All` with the selected operating system's definitions, merges
entries with the same package name, and deduplicates file paths.

Only implementation files enter the ZIP. They can include executables, scripts, and required
support files; nested relative paths are preserved. Executable filenames use the target platform's
native suffix, regardless of how they are written in shared metadata. Manifest sources are processed separately
and are not required merely to assemble the ZIP. Official production package definitions MUST
exclude test resources.

For publication, all declared implementation files and manifest sources MUST be available for
every requested platform. Producers validate the complete publication input before uploading.
Artifact-directory organization and build tools are implementation choices.

Each publication input MUST describe one resource, in JSON or YAML; bundles are rejected.
Build preparation can generate individual resource files from source bundles, preserving the
bundles for ordinary distribution and retaining resource properties and existing authors.
Actions and extensions in bundles are not emitted as resources. Package definitions select the
individual files appropriate for each operating system. The same resource version and platform
MUST NOT be advertised by multiple packages; Windows-only ACL manifests are excluded from Unix
package definitions even when their implementation is shared with File Content.

### Registry addressing

Package repositories use:

```text
<registry>/dsc/resource/<lowercase-package-name>
```

The registry host is a deployment choice. Azure Container Registry is one supported deployment,
not a requirement on consumer languages or implementations.

The resource slug is the lowercase resource name after `/`. For example, RegistryList has the
slug `registrylist`. Within its package repository:

- `<slug>-<version>` identifies the resource's OCI index.
- `<slug>-<version>-<platform>` identifies a platform-specific child manifest.
- `<repository>@<digest>` identifies immutable content.

For RegistryList version `1.0.0`, the index tag is `registrylist-1.0.0` and a child tag is
`registrylist-1.0.0-windows-amd64`.

Generated tags MUST satisfy registry tag syntax and the 128-character limit. Different resource
types that produce the same resource-version tag in a package are a publication error. Producers
MUST reject invalid mappings rather than overwrite another resource's identity.

This initial mapping uses the version literally. A semantic version containing build metadata
with `+` cannot form a valid OCI tag and MUST be rejected by publication. Encoding such versions
is a future naming decision; silently rewriting the version is not permitted.

Tags are mutable references. Consumers MUST use a resolved child digest for acquisition so that
retagging an index or child during a transfer does not change the selected content.

### Platform artifacts and shared blobs

Each platform artifact is an OCI image manifest with:

- `schemaVersion` equal to `2`.
- `mediaType` equal to `application/vnd.oci.image.manifest.v1+json`.
- `artifactType` equal to `application/vnd.microsoft.dsc.resource.v1+zip`.
- A valid OCI configuration descriptor; the OCI empty JSON configuration is suitable.
- One DSC resource manifest layer with media type `application/json`.
- One implementation ZIP layer with media type `application/zip`.

These artifacts are not container filesystem images and do not require a container runtime.
Consumers locate file layers by media type, not array position, and MUST reject missing or
ambiguous resource manifest or ZIP layers. Additional OCI metadata does not authorize downloading
or executing additional files.

Each file layer descriptor includes its SHA-256 digest, exact byte size, and
`org.opencontainers.image.title` filename annotation. The resource manifest filename is
`<slug>.dsc.resource.json` for command resources and `<slug>.dsc.adaptedResource.json` for
adapted resources, preserving the filename distinction used by DSC's manifest loaders.
The ZIP filename is `<package-name>.zip`.

Both the platform manifest and its resource index carry these annotations:

- `com.microsoft.dsc.resource.id`: full resource type.
- `com.microsoft.dsc.version`: resource version.
- `com.microsoft.dsc.kind`: lowercase resource kind, defaulting to `resource`.
- `org.opencontainers.image.created`: UTC creation time in RFC 3339 format.
- `com.microsoft.dsc.publisher`: the resource's author, when present.

File layer annotations additionally carry `com.microsoft.dsc.resource.id` and
`com.microsoft.dsc.resource.version`. The distinct version annotation names are part of the
proposed wire format.

The same ZIP descriptor can appear in multiple resource-specific platform manifests. Registry
content addressing permits identical ZIP bytes to share a blob in the package repository.
Identical filenames alone do not establish identical content.

### Resource indexes

An OCI image index groups the platform artifacts for one full resource type and version. It uses
media type `application/vnd.oci.image.index.v1+json`, schema version `2`, the DSC artifact type,
and the resource annotations described above.

Each child descriptor records media type, digest, byte size, and platform. This profile requires
at least one child and an unambiguous descriptor for each published platform.

The following example is schematic; the child digest and size do not describe an actual upload:

```json
{
  "schemaVersion": 2,
  "mediaType": "application/vnd.oci.image.index.v1+json",
  "artifactType": "application/vnd.microsoft.dsc.resource.v1+zip",
  "annotations": {
    "com.microsoft.dsc.resource.id": "Microsoft.Windows/RegistryList",
    "com.microsoft.dsc.version": "1.0.0",
    "com.microsoft.dsc.kind": "resource",
    "com.microsoft.dsc.publisher": "Microsoft Corporation",
    "org.opencontainers.image.created": "2026-11-21T23:00:00Z"
  },
  "manifests": [
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "digest": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
      "size": 2048,
      "platform": {
        "os": "windows",
        "architecture": "amd64"
      }
    }
  ]
}
```

### Publication flow

1. Resolve the package, target platforms, implementation files, and individual resource manifests.
   Validate resource identities, semantic versions, generated names, and all required input files.
   Duplicate resource-platform entries or conflicting resource tags fail preflight.
2. Assemble one implementation ZIP per target platform. Prepare each resource's JSON manifest and
   annotations without modifying the original manifest sources.
3. Upload the ZIP, individual resource manifest, and required configuration blobs. Upload each
   resource-specific platform manifest and establish its child tag.
4. Obtain and validate each uploaded platform manifest's descriptor. Assemble the resource index
   from those descriptors, including their platforms.
5. Publish the resource-version index only after its requested children are available.

An index write is the discoverability boundary for that resource version's platform set. Failed
publication can leave uploaded blobs or child tags, but MUST NOT create an index referencing
children that failed to upload. There is no package-wide transaction across different resources'
indexes.

Writing an existing index tag replaces that index; automatic append or merge is not specified.
To retain previously published platforms, a publisher or coordinating release pipeline must
assemble the complete desired platform set. Concurrent publication to the same resource-version
tag requires coordination to prevent lost platform descriptors.

### Authentication and authorization

Transport uses HTTPS and the registry's authentication challenge and credential exchange.
Credential providers are separate from artifact processing. A provider can supply existing
registry credentials or acquire a short-lived registry token from an authenticated identity.
For Azure Container Registry, this can be an ACR token acquired through Azure identity services.
No particular SDK, external executable, or language runtime is required.

Publishing requires blob and manifest write permissions. Discovery requires repository and tag
listing plus manifest read access. Acquisition requires manifest and blob read access. Reads may
be anonymous where registry policy permits; otherwise consumers use available registry credentials.

Tokens MUST NOT enter artifacts, cache metadata, logs, or ordinary temporary files. Authentication
and authorization failures are errors, not evidence that a resource has no matching versions.

### Metadata-only discovery

The baseline resolver does not assume that replacing `/` in a resource type identifies its
package repository. It performs the following steps:

1. Enumerate accessible repositories and restrict candidates to `dsc/resource/`.
2. Enumerate their tags, following pagination and deduplicating entries.
3. Consider tags with the requested resource slug prefix. An exact version filter further limits
   candidates to the exact resource-version index tag.
4. Read candidate OCI documents. Ignore platform child manifests and unrelated artifact types.
   Match the full `com.microsoft.dsc.resource.id` annotation, not just the slug.
5. Validate matching indexes and their descriptors, apply version and platform filters, and
   produce one discovery record per matching platform.

Repository enumeration can use a registry catalog or a registry-provider listing API. Catalog
enumeration is not universally available through the OCI Distribution Specification. A registry
without a usable repository listing requires another package mapping source; failure to list
repositories MUST NOT be represented as an empty search result.

A discovery record includes full resource type, version, platform, child descriptor digest,
media type and size, annotations, resource index reference, and a digest-pinned child reference.
The package name is derived from the repository or the later ZIP title, not from the requested
resource type alone.

Omitted search filters return all matching versions or platforms, including prereleases.
Discovery reads OCI metadata only; it does not fetch the child file layers. A valid completed
search can return no matches. Malformed matching metadata, transport failures, or permission
failures MUST be reported distinctly.

### Acquisition and selection

Acquisition resolves one resource artifact before modifying the cache. With an explicit version,
the version match is exact. Without a version, the resolver filters to the requested platform,
excludes prereleases, and chooses the highest version using semantic version precedence rather
than lexical ordering. Without an explicit platform, it uses the host platform.

No stable match, an unsupported platform, or multiple matching artifacts at the selected version
is an error. Consumers MUST NOT choose an arbitrary repository, prerelease, or architecture as a
fallback.

The consumer fetches the platform OCI manifest by the selected digest and verifies its identity,
version, artifact type, and file descriptors. It downloads only the required resource manifest
and ZIP blobs by digest. SHA-256 hashes and byte sizes MUST match their descriptors. The downloaded
DSC resource manifest's full type and version MUST match the selected artifact.

Acquisition does not imply publisher trust. Digest verification establishes content integrity,
not the authenticity of an author annotation. Signature verification and organizational trust
policy are separate concerns.

### Versioned per-user cache

The cache root is the user's local application data directory plus `dsc` on Windows, normally
`%LOCALAPPDATA%\dsc`. On non-Windows systems it is `.dsc` in the user's home directory.
The destination layout is:

```text
<cache-root>/resourceCache/<package-name>/<package-version>/
```

The package name comes from the ZIP filename without `.zip`. The version directory contains
the extracted implementation files, individual downloaded resource manifests, and local cache
metadata. Multiple versions MUST coexist without acquisition of one deleting another.

Filenames and versions MUST be safe local path components. ZIP and manifest titles use the
expected suffixes and only ASCII letters, digits, periods, underscores, and hyphens, beginning
with a letter or digit. Versions MUST parse as semantic versions. Consumers MUST reject path
traversal, absolute paths, and linked package or version directories that redirect cache writes.
ZIP entries MUST remain inside the extraction destination.

Local metadata is stored as `.dsc-cache.json` in the version directory with these fields:

- `ZipDigest`: SHA-256 digest of the installed implementation ZIP.
- `Version`: selected resource version.
- `Platform`: installed platform.
- `ManifestDigests`: mapping from downloaded resource manifest filenames to their blob digests.
- `Files`: relative paths of implementation files extracted from the ZIP.

Consumers reuse an implementation only when ZIP digest, version, and platform match and all
recorded implementation files exist. A resource manifest is reusable only when its recorded
digest matches the requested layer and the cached file still hashes to that digest. A missing
implementation file requires reconstruction; a missing or changed resource manifest requires
that manifest to be downloaded again.

Resources sharing the same ZIP, version, and platform share an extracted implementation. Adding
another resource manifest preserves existing manifests for that matching package. Changing the
ZIP or platform reconstructs only the selected version directory and discards its stale contents.
Other version directories are unaffected.

This layout holds one platform per package version at a time. It does not provide side-by-side
architectures within the same version directory.

### Cache installation and failure handling

Downloads and ZIP extraction occur in a uniquely owned staging directory on the cache filesystem.
All downloaded data and resource identities are validated before a new installation is exposed.
Temporary ZIP files need not remain in the final version directory.

For a new or reconstructed implementation, the consumer prepares the complete version directory
and its metadata before installation. Replacement preserves the previous directory until the
prepared directory is installed successfully, restoring it if installation fails. If recovery
cannot complete, retained recovery data and its location MUST be reported.

When missing implementation files require repair but the ZIP digest, version, and platform still
match, consumers MUST retain sibling resource manifests whose files match their recorded digests.
Missing or corrupted siblings are reported and excluded from the repaired cache. A changed ZIP,
version, or platform does not authorize retaining manifests from the previous installation.

For a matching shared implementation, only the required resource manifest and its metadata entry
need updating, but these updates MUST preserve the live installation until both are ready.
Staging a copy of the matching directory allows the same replacement and recovery mechanism
to apply without downloading the implementation again. Success is reported only after both
are installed. Writers targeting the same version directory MUST coordinate before reading
cache state and through installation and rollback to avoid losing shared manifests or publishing
inconsistent metadata; the locking or transactional mechanism is implementation-defined.

Consumers clean up only staging directories they own. Failed downloads, invalid archives,
digest mismatches, malformed manifests, and filesystem errors MUST NOT yield successful
installation results. They MUST NOT delete unrelated versions or broad cache roots.

Cache reuse follows remote artifact resolution. A fully offline resolver, automatic cache
eviction, dependency installation, and automatic execution are outside this proposal.

## Alternate Proposals and Considerations

### One artifact containing all manifests

A package-level artifact could include every resource manifest with its ZIP. This simplifies
publication but requires consumers to retrieve or maintain an additional resource-to-package
catalog to discover resources without downloading manifest blobs. Per-resource indexes preserve
independent discovery while still sharing implementation blobs.

### One repository and ZIP per resource

Deriving a repository directly from every resource type simplifies lookup, but splits the
implementation package relationship across repositories and publication workflows. The proposed
package repositories keep related resources together; consumers validate full resource identities
to disambiguate similar names.

### A dedicated search catalog

A catalog mapping full resource types to repositories could reduce repository and tag enumeration
and support registries without listing access. It introduces another artifact or service whose
updates must remain consistent with publication. Such a catalog can be added later without
changing the platform artifact and resource index formats.

### Alternative cache layouts

A flat package directory cannot retain multiple versions. A digest-only cache permits stronger
deduplication but obscures resource package and version identities. Adding a platform directory
would permit simultaneous architectures, but differs from the proposed
`resourceCache/<package-name>/<package-version>` layout and is deferred.

### Publication evolution and trust

Automatic platform-index merging, immutable release-tag policy, encoding semantic version build
metadata, signature verification, and richer publisher trust policy require further decisions.
This proposal makes the current replacement and integrity boundaries explicit without treating
digest verification or publisher annotations as signature validation.

## Related work items

No GitHub issues, discussions, or pull requests are linked at the time of drafting.

[distribution]: https://github.com/opencontainers/distribution-spec/blob/v1.1.0/spec.md
[manifest]: https://github.com/opencontainers/image-spec/blob/v1.1.0/manifest.md
[index]: https://github.com/opencontainers/image-spec/blob/v1.1.0/image-index.md
[semver]: https://semver.org/spec/v2.0.0.html
