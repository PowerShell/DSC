## ORAS Discovery for DSC Artifacts

This document defines how DSC artifacts are discovered from an OCI-compliant container registry by using ORAS conventions.

The primary objective is to make DSC artifacts easy to find, filter, and install across environments by using a consistent artifact model.

## Purpose

- Provide a clear discovery contract for DSC artifacts in OCI registries.
- Standardize metadata and artifact typing so clients can query reliably.
- Enable platform-aware discovery and selection.
- Support implementation of the DSC discovery extension in Rust.

## Scope

This guidance applies to discovery and install preparation for these DSC artifact families:

- DSC Schemas
- DSC Configurations
- DSC Resources
- DSC Extensions
- DSC Adapters

## Discovery Model

- A registry repository stores one logical resource type.
- A tag identifies a specific version or release of that resource.
- `artifactType` identifies the DSC artifact kind.
- OCI manifest `layers` contain the distributable payload.
- Installers pull selected layers and extract payloads locally.
- OCI index manifests are used to group related manifests and support OS/architecture filtering.

## Manifest Schemas (Normative Samples)

Do not alter the sample payload structure when implementing parsers or validators. These samples define the expected shape and annotations.

### DSC Schema Artifact sample OCI manifest

```json
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "artifactType": "application/vnd.microsoft.dsc.schema.v1+json",
    "config": {
        "mediaType": "application/vnd.oci.empty.v1+json",
        "digest": "sha256:44136fa355b3678a1146a... (example)",
        "size": 2
    },
    "layers": [
        {
            "mediaType": "application/schema+json",
            "digest": "sha256:aaaaaaaaaaaaaaaaaaaaaaaa...",
            "size": 12345,
            "annotations": {
                "org.opencontainers.image.title": "example.dsc.resource.json",
                "org.opencontainers.image.description": "DSC schema for Example"
            }
        }
    ],
    "annotations": {
        "org.opencontainers.image.created": "2026-02-28T15:16:17Z",
        "com.microsoft.dsc.kind": "schema",
        "com.microsoft.dsc.name": "Microsoft.DSC/Example",
        "com.microsoft.dsc.version": "1.2.3"
    }
}
```

### DSC Resource Artifact sample OCI manifest

```json
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "artifactType": "application/vnd.microsoft.dsc.resource.v1+tar+gzip",
    "config": {
        "mediaType": "application/vnd.oci.empty.v1+json",
        "digest": "sha256:44136fa355b3678a1146a... (example)",
        "size": 2
    },
    "layers": [
        {
            "mediaType": "application/tar+gzip",
            "digest": "sha256:bbbbbbbbbbbbbbbbbbbbbbbb...",
            "size": 987654,
            "annotations": {
                "org.opencontainers.image.title": "resource.tar.gz",
                "com.microsoft.dsc.resource.id": "Contoso/MyResource",
                "com.microsoft.dsc.resource.version": "2.0.0"
            }
        }
    ],
    "annotations": {
        "org.opencontainers.image.created": "2026-02-28T15:16:17Z",
        "com.microsoft.dsc.kind": "resource",
        "com.microsoft.dsc.publisher": "Contoso",
        "com.microsoft.dsc.resource.id": "Contoso/MyResource",
        "com.microsoft.dsc.version": "2.0.0"
    }
}
```

### DSC Configuration Artifact sample OCI manifest

```json
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.manifest.v1+json",
    "artifactType": "application/vnd.microsoft.dsc.configuration.v1+yaml",
    "config": {
        "mediaType": "application/vnd.oci.empty.v1+json",
        "digest": "sha256:44136fa355b3678a1146a... (example)",
        "size": 2
    },
    "layers": [
        {
            "mediaType": "text/yaml",
            "digest": "sha256:cccccccccccccccccccccccc...",
            "size": 4321,
            "annotations": {
                "org.opencontainers.image.title": "configuration.dsc.yml",
                "com.microsoft.dsc.configuration.name": "Baseline-WebServer"
            }
        }
    ],
    "annotations": {
        "org.opencontainers.image.created": "2026-02-28T15:16:17Z",
        "com.microsoft.dsc.kind": "configuration",
        "com.microsoft.dsc.name": "Baseline-WebServer",
        "com.microsoft.dsc.version": "1.0.0"
    }
}
```

### OCI index file sample

The index ties together related artifacts and enables platform filtering using platform.os and platform.architecture. [github.com], [specs.open...ainers.org]

```json
{
    "schemaVersion": 2,
    "mediaType": "application/vnd.oci.image.index.v1+json",
    "artifactType": "application/vnd.microsoft.dsc.bundle.v1+json",
    "manifests": [
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "digest": "sha256:111111111111111111111111...",
            "size": 1500,
            "platform": {
                "os": "linux",
                "architecture": "amd64"
            },
            "annotations": {
                "com.microsoft.dsc.role": "resource",
                "com.microsoft.dsc.resource.id": "Contoso/MyResource",
                "com.microsoft.dsc.version": "2.0.0"
            }
        },
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "digest": "sha256:222222222222222222222222...",
            "size": 1200,
            "platform": {
                "os": "windows",
                "architecture": "amd64"
            },
            "annotations": {
                "com.microsoft.dsc.role": "resource",
                "com.microsoft.dsc.resource.id": "Contoso/MyResource",
                "com.microsoft.dsc.version": "2.0.0"
            }
        },
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "digest": "sha256:333333333333333333333333...",
            "size": 900,
            "annotations": {
                "com.microsoft.dsc.role": "schema",
                "com.microsoft.dsc.name": "MySchema",
                "com.microsoft.dsc.version": "1.2.3"
            }
        },
        {
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "digest": "sha256:444444444444444444444444...",
            "size": 950,
            "annotations": {
                "com.microsoft.dsc.role": "configuration",
                "com.microsoft.dsc.name": "Baseline-WebServer",
                "com.microsoft.dsc.version": "1.0.0"
            }
        }
    ],
    "annotations": {
        "org.opencontainers.image.created": "2026-02-28T15:16:17Z",
        "com.microsoft.dsc.bundle.name": "Baseline-WebServer",
        "com.microsoft.dsc.bundle.version": "1.0.0"
    }
}
```

### Discovery extension design details

- One repository stores one resource type.
- The resource tag identifies a specific resource version.
- `artifactType` distinguishes resource kinds.
- OCI layers store the artifact payload.
- Resource payloads are packaged as `tar.gz`.
- Installation pulls selected layers and extracts contents.


#### Phase 1: Discovery

- Register one or more registry repositories (for example, ACR repositories) with the discovery extension.
- Enumerate candidate artifacts from configured repositories.
- Filter artifacts by user-provided criteria, including artifact type and version.
- Support discovery of one artifact or multiple artifacts in a single request.

## Required Discovery Inputs

The extension should accept:

- Registry reference (host/repository)
- Optional tag or version selector
- Artifact kind filter (schema, resource, configuration, extension, adapter)
- Optional platform filter (`os`, `architecture`)

## Expected Discovery Output

For each matching artifact, return enough metadata to support install and display:

- Repository
- Tag
- Digest
- Artifact type
- Version annotations
- Resource identity annotations (when present)
- Platform metadata (when present)
- Layer descriptors needed for download and extraction

## Validation Rules

- Reject manifests that do not include required OCI fields (`schemaVersion`, `mediaType`, `config`, `layers`).
- Validate that `artifactType` aligns with the expected DSC artifact category.
- Validate required DSC annotations for the selected artifact kind.
- If an OCI index is used, prefer manifests that match requested platform filters.
- If no platform match exists, return a clear "not found for platform" result.

## Error Handling Guidance

- Distinguish discovery failures from install failures.
- Return actionable error categories:
    - Registry/authentication error
    - Manifest parse/validation error
    - Artifact not found
    - Platform variant not found
- Include repository/tag/digest context in errors when available.

## Rust Implementation Notes

- Model manifests and indexes with strongly typed structs.
- Treat unknown annotations as forward-compatible metadata.
- Keep filtering deterministic: apply kind/version filters first, then platform.
- Prefer pure discovery logic separated from download/extract logic.
- Add tests for:
    - single-manifest discovery
    - index-based platform selection
    - annotation validation
    - no-match and malformed-manifest scenarios

## Non-Goals (Phase 1)

- Publishing artifacts to registries
- Registry mutation workflows
- Full install execution beyond selecting and returning artifact layers

## Phase Progression

- Phase 1: discovery registration, scan, filter, and selection
- Later phases: install orchestration, caching, signature/trust verification, richer policy controls