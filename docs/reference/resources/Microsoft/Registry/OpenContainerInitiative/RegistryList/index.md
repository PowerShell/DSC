---
description: Microsoft.Registry.OpenContainerInitiative/RegistryList resource reference documentation
ms.date:     11/22/2026
ms.topic:    reference
title:       Microsoft.Registry.OpenContainerInitiative/RegistryList
---

# Microsoft.Registry.OpenContainerInitiative/RegistryList

## Synopsis

Manages the registries used by `Microsoft.Registry.OpenContainerInitiative/Discover`.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [OCI]
Author     : Microsoft Corporation
```

## Instance definition syntax

```yaml
resources:
  - name: Configure registry discovery
    type: Microsoft.Registry.OpenContainerInitiative/RegistryList
    properties:
      # Required for set; replaces the entire selected list.
      registries:
        - public:
            location: registry.example.com
        - private:
            location: private.example.com
            authentication:
              secretName: registry-access-token
              secretVault: registry-vault
      # Optional selectors.
      scope: currentUser
      policy: false
```

## Description

The resource reads, tests, replaces, and exports one registry settings file. `scope` defaults to
`currentUser`; `policy` defaults to `false`. Get and export return the selected file's list,
not the merged effective discovery configuration. Missing files return an empty list; invalid
JSON, invalid entries, and inaccessible files are errors. Set replaces the selected list
atomically, and an empty `registries` array clears it.

Test compares the entire selected registry list, including array order and optional
authentication fields. Removing `authentication` or `secretVault` from the desired list is a
change that set must apply. Omitting `registries` tests only the selected scope and policy,
without requesting a list change.

Export adds the top-level, read-only `_name` property with the value `OCI Registries`.
DSC uses this value as the exported configuration instance's name and removes it from the
instance properties. Get and set don't return `_name`, and it isn't persisted in settings files.

Preference files are named `Microsoft.Registry.OpenContainerInitaitive.settings.json`.
The spelling `Initaitive` in the filename is intentional. Policy files are named
`Microsoft.Registry.OpenContainerInitaitive.policy.json`. Both files contain only the
`registries` property, not the `scope` or `policy` selectors.

Current-user files reside in `%LOCALAPPDATA%\dsc` on Windows and `~/.dsc` elsewhere.
All-users files reside in `%ProgramData%\dsc` on Windows,
`/Library/Application Support/dsc` on macOS, and `/etc/dsc` on Linux.
Policy is supported only with `scope: allUsers`. Writing either all-users file requires
administrator privileges. Machine policy files must remain protected by administrator-controlled
filesystem permissions.

The Discover extension enumerates every distinct friendly name across all three files.
For an identical, case-sensitive friendly name, precedence is:

1. All-users policy.
2. Current-user preference.
3. All-users preference.

A policy entry cannot be overridden by user or machine preferences. Policy does not prevent
adding other friendly names. Current-user preferences can override machine preferences or add
additional registries.

When DSC performs resource discovery, including `dsc resource list`, the extension enumerates
registry repositories under `dsc/resource/`, paginates tags, and reads the resource indexes
defined by the OCI resource distribution RFC. It lists all published versions, including
prereleases, that have a descriptor for the host operating system and architecture.
It retrieves the digest-pinned platform document and its individual JSON DSC resource manifest,
verifying SHA-256 digests, sizes, and resource identity. It does **not** download implementation
ZIPs, install packages, or modify resource search paths. Resources still need their implementations
installed before invocation.

Remote definitions are imported only as ordinary DSC resource manifests, without evaluating
conditions or executing schema commands. Action, adapted-resource, and manifest-list payloads
are rejected, and remote definitions cannot supply trusted local resource paths.

Remote list results include `registry` set to the friendly name. Locally discovered resources
include `registry: local`. An installed local resource takes precedence over a remote resource
with the same type and version, consistently for listing and invocation.
Registries must permit repository catalog enumeration. An unavailable registry or failed
credential lookup produces a warning identifying that registry; discovery continues with the
remaining registries and retains earlier results. Invalid remote manifests are also reported as
warnings without discarding valid results. Invalid overall settings remain an extension error,
which DSC reports while preserving local discovery.

When an entry specifies `authentication`, the extension requests its token through the host's
reverse JSON-RPC `secret.get` service. The provider must return a nonempty `secureString`.
The token is sent as a bearer credential to the registry. Tokens are never written to the settings
file or emitted in diagnostic output. Without authentication, requests are anonymous, including
anonymous bearer-token negotiation when required by a public registry.
Registry locations use HTTPS by default; explicit HTTP is permitted only for loopback registries.

> [!NOTE]
> This resource and the discovery extension are installed with DSC on all platforms.
> Updating DSC updates both components.

## Requirements

- Administrator privileges to write all-users preferences or policy.
- A loaded Secret extension for authenticated registries.
- Registry repository-listing permissions and the resource distribution RFC artifact layout.

## Capabilities

The resource has the following capabilities:

- `get` - Retrieve the selected settings list.
- `set` - Replace the selected settings list.
- `test` - Compare the complete selected settings list with the desired list.
- `export` - Export the selected settings list as one resource instance.

For more information, see [DSC resource capabilities][01].

## Examples

1. [Configure registry preferences and policy][02].

## Properties

The resource has no key properties. `registries` is required for set and optional for get, test,
and export.
The optional instance properties are `scope`, `policy`, and `registries`.
The read-only `_name` property is returned only by export.

### _name

<details><summary>Expand for <code>_name</code> property metadata</summary>

```yaml
Type        : [string, 'null']
IsRequired  : false
IsKey       : false
IsReadOnly  : true
IsWriteOnly : false
```

</details>

Export returns `OCI Registries` as the suggested configuration instance name.

### scope

<details><summary>Expand for <code>scope</code> property metadata</summary>

```yaml
Type         : string
IsRequired   : false
IsKey        : false
IsReadOnly   : false
IsWriteOnly  : false
DefaultValue : currentUser
ValidValues  : [currentUser, allUsers]
```

</details>

Selects the current user's list or the machine-wide list.

### policy

<details><summary>Expand for <code>policy</code> property metadata</summary>

```yaml
Type         : boolean
IsRequired   : false
IsKey        : false
IsReadOnly   : false
IsWriteOnly  : false
DefaultValue : false
```

</details>

Selects machine policy instead of preferences. `true` requires `scope: allUsers`.

### registries

<details><summary>Expand for <code>registries</code> property metadata</summary>

```yaml
Type        : array
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

Required for set. Each array item contains exactly one friendly-name property whose value is a
registry object. Friendly names must be nonempty and unique within the selected file;
`local` is reserved, regardless of letter casing, because DSC uses it for locally discovered
resources. Using it as a registry friendly name returns a reserved-name error.

#### location

<details><summary>Expand for <code>location</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : true
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

A registry hostname with optional port and `https://` prefix. Paths, user information, queries,
and fragments are not permitted.

#### authentication

<details><summary>Expand for <code>authentication</code> property metadata</summary>

```yaml
Type        : object
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

An optional secret reference. Omit the object for anonymous access.

#### secretName

<details><summary>Expand for <code>secretName</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : true
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The nonempty name of the secret in `authentication`.

#### secretVault

<details><summary>Expand for <code>secretVault</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The optional nonempty vault name. If omitted, the host resolves the secret without a vault filter.

## Instance validating schema

The resource provides its schema through its manifest's schema command. Retrieve the full schema
with `dsc resource schema -r Microsoft.Registry.OpenContainerInitiative/RegistryList`.
The following equivalent structure omits descriptive schema metadata:

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "_name": {"type": ["string", "null"], "readOnly": true},
    "scope": {"type": "string", "enum": ["currentUser", "allUsers"], "default": "currentUser"},
    "policy": {"type": "boolean", "default": false},
    "registries": {
      "type": ["array", "null"],
      "items": {
        "type": "object",
        "additionalProperties": {
          "type": "object",
          "additionalProperties": false,
          "required": ["location"],
          "properties": {
            "location": {"type": "string"},
            "authentication": {
              "type": ["object", "null"],
              "additionalProperties": false,
              "required": ["secretName"],
              "properties": {
                "secretName": {"type": "string"},
                "secretVault": {"type": ["string", "null"]}
              }
            }
          }
        }
      }
    }
  }
}
```

The operation also validates friendly-name uniqueness, singleton entries, registry locations,
nonempty secret references, and the policy/scope combination.

## Exit codes

The resource returns the following exit codes:

- [0](#exit-code-0) - Success.
- [1](#exit-code-1) - Invalid arguments.
- [2](#exit-code-2) - Invalid input.
- [3](#exit-code-3) - Settings or registry operation failed.

### Exit code 0

The operation completed successfully.

### Exit code 1

Invalid command-line arguments. The executable writes an argument error to stderr.

### Exit code 2

Invalid JSON input or registry settings. The executable writes a JSON error to stderr.

### Exit code 3

An operation failed, such as accessing a settings file, insufficient privileges, fetching a secret,
or contacting a registry. The executable writes a JSON error to stderr. Discovery invocation
failures are returned to DSC as JSON-RPC errors.

## See also

- [Discover extension operation][03].
- [RFC: Resource packages and OCI resource distribution][04].

<!-- Link definitions -->
[01]: ../../../../../../concepts/resources/capabilities.md
[02]: ./examples/configure-registries.md
[03]: ../../../../../schemas/extension/manifest/discover.md
[04]: https://github.com/PowerShell/DSC/pull/1826
