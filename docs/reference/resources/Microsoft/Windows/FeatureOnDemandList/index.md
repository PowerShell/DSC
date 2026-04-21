---
description: Microsoft.Windows/FeatureOnDemandList resource reference documentation
ms.date:     04/21/2026
ms.topic:    reference
title:       Microsoft.Windows/FeatureOnDemandList
---

# Microsoft.Windows/FeatureOnDemandList

## Synopsis

Manage Windows features on demand (capabilities) using the DISM API.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows, dism, capability, featureondemand, fod]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/FeatureOnDemandList
    properties:
      # Required properties
      capabilities:
        - identity: string
          # Instance properties
          state: Installed | NotPresent
```

## Description

The `Microsoft.Windows/FeatureOnDemandList` resource enables you to idempotently manage Windows
features on demand (also known as capabilities) using the DISM API. Features on demand are optional
Windows components that are not part of the base OS image and may need to be downloaded from
Windows Update or a local source before use. Examples include language packs, accessibility tools,
the OpenSSH client and server, and developer tools like the RSAT (Remote Server Administration
Tools) suite.

The resource can:

- Retrieve the current state of one or more capabilities by identity.
- Install capabilities (`Installed`), downloading them from Windows Update if necessary.
- Remove capabilities from the system (`NotPresent`).
- Export a list of all capabilities on the system, optionally filtered by identity, state, display
  name, or description.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- All operations require an elevated (administrator) process context.
- Installing capabilities may require internet access or a configured Windows Update / WSUS source.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of one or more capability
  instances.
- `set` - You can use the resource to enforce the desired state for one or more capability
  instances.
- `export` - You can use the resource to enumerate all capabilities on the system, with optional
  filtering.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][01].

## Examples

1. [Get feature on demand state][02] - Shows how to retrieve the current state of a Windows
   capability.
1. [Install and remove features on demand][03] - Shows how to install and remove Windows
   capabilities using the `dsc resource set` command.
1. [Export features on demand][04] - Shows how to enumerate all capabilities on the system, with
   and without filters.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource.

  - [capabilities](#capabilities) - An array of capability entries.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][05].

  - [_restartRequired](#_restartrequired) - Indicates that a system restart is required to complete
    the state change.

### capabilities

<details><summary>Expand for <code>capabilities</code> property metadata</summary>

```yaml
Type       : array
IsRequired : true
IsKey      : false
IsReadOnly : false
```

</details>

An array of capability entries. Each entry is an object describing a Windows capability (Feature on
Demand). For the **Get** operation, each entry must specify [`identity`](#identity). For the **Set**
operation, each entry must specify both [`identity`](#identity) and [`state`](#state). For the
**Export** operation, the array is optional and each entry can filter results using
[`identity`](#identity), [`state`](#state), [`displayName`](#displayname), or
[`description`](#description) with wildcard support.

Each entry in `capabilities` has the following properties:

- [identity](#identity) - The identity string of the capability.
- [_exist](#_exist) - Indicates whether the capability is recognized by DISM.
- [state](#state) - The current or desired state of the capability.
- [displayName](#displayname) - The display name of the capability.
- [description](#description) - The description of the capability.
- [downloadSize](#downloadsize) - The download size of the capability in bytes.
- [installSize](#installsize) - The install size of the capability in bytes.

#### identity

<details><summary>Expand for <code>capabilities[*].identity</code> property metadata</summary>

```yaml
Type       : string
IsRequired : true (get, set) / false (export)
IsKey      : false
IsReadOnly : false
```

</details>

The identity string that uniquely identifies the Windows capability. For **Get** and **Set**
operations, this property is required for each entry. For **Export** operations, it's optional and
supports wildcard (`*`) patterns for case-insensitive filtering.

Capability identities typically follow the format `CapabilityName~~~~LanguageTag~Version`, for
example `OpenSSH.Client~~~~0.0.1.0` or `Language.Basic~~~en-US~0.0.1.0`.

Use the `dism /Online /Get-Capabilities` command to list available capability identities on your
system.

#### _exist

<details><summary>Expand for <code>capabilities[*]._exist</code> property metadata</summary>

```yaml
Type       : boolean
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

Indicates whether the capability exists on the system. The resource sets this property to `false`
in the **Get** response when the requested `identity` is not recognized by DISM. When `_exist` is
`false`, the `state`, `displayName`, `description`, `downloadSize`, and `installSize` properties
are not returned.

#### state

<details><summary>Expand for <code>capabilities[*].state</code> property metadata</summary>

```yaml
Type         : string
IsRequired   : true (set) / false (get, export)
IsKey        : false
IsReadOnly   : false (set input) / true (get/export output)
ValidValues  : [NotPresent, UninstallPending, Staged, Removed, Installed,
                InstallPending, Superseded, PartiallyInstalled]
SetValues    : [Installed, NotPresent]
```

</details>

The state of the capability. **Get** and **Export** operations return one of the eight DISM
capability state values. **Set** operations accept only the following two values as desired state:

| Value        | Description                                                                    |
|:-------------|:-------------------------------------------------------------------------------|
| `Installed`  | The capability is installed. The resource installs it if not already present.  |
| `NotPresent` | The capability is removed from the system.                                     |

The following table describes all possible state values returned by **Get** and **Export**:

| Value                | Description                                                               |
|:---------------------|:--------------------------------------------------------------------------|
| `NotPresent`         | The capability is not installed and not staged.                           |
| `UninstallPending`   | A removal operation is pending, requiring a restart to complete.          |
| `Staged`             | The capability payload is on disk but the capability is not installed.    |
| `Removed`            | The capability has been removed.                                          |
| `Installed`          | The capability is fully installed and operational.                        |
| `InstallPending`     | An install operation is pending, requiring a restart to complete.         |
| `Superseded`         | The capability has been replaced by another component.                    |
| `PartiallyInstalled` | The capability is only partially installed.                               |

#### displayName

<details><summary>Expand for <code>capabilities[*].displayName</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

The human-readable display name of the capability. This property is returned by **Get** and
**Export** operations. For **Export** operations, you can specify this property as a filter value
with wildcard (`*`) support for case-insensitive matching.

#### description

<details><summary>Expand for <code>capabilities[*].description</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

A brief description of the capability. This property is returned by **Get** and **Export**
operations. For **Export** operations, you can specify this property as a filter value with
wildcard (`*`) support for case-insensitive matching.

#### downloadSize

<details><summary>Expand for <code>capabilities[*].downloadSize</code> property metadata</summary>

```yaml
Type       : integer
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

The size in bytes that must be downloaded to install the capability. This property is returned by
**Get** and **Export** operations.

#### installSize

<details><summary>Expand for <code>capabilities[*].installSize</code> property metadata</summary>

```yaml
Type       : integer
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

The size in bytes that the capability occupies on disk after installation. This property is returned
by **Get** and **Export** operations.

### _restartRequired

<details><summary>Expand for <code>_restartRequired</code> property metadata</summary>

```yaml
Type       : array
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

Returned at the top level of the **Set** operation response when DISM reports that a system restart
is required to complete the requested state changes. Each entry in the array is an object with a
`system` property containing the name of the computer.

When no restart is required, this property is omitted from the response.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["capabilities"],
  "additionalProperties": false,
  "properties": {
    "_restartRequired": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "capabilities": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "identity": { "type": "string" },
          "_exist": { "type": "boolean" },
          "state": {
            "type": "string",
            "enum": [
              "NotPresent", "UninstallPending", "Staged", "Removed",
              "Installed", "InstallPending", "Superseded", "PartiallyInstalled"
            ]
          },
          "displayName": { "type": "string" },
          "description": { "type": "string" },
          "downloadSize": { "type": "integer" },
          "installSize": { "type": "integer" }
        }
      }
    }
  }
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Error

### Exit code 0

Indicates the resource operation completed without errors. The resource writes the result JSON to
stdout.

### Exit code 1

Indicates the resource operation failed. The resource writes a descriptive error message to stderr.
Common causes include:

- The `capabilities` array is empty.
- The `identity` property is missing from a capability entry in a **Get** or **Set** operation.
- The `state` property is missing from a capability entry in a **Set** operation.
- The desired `state` value is not one of the accepted **Set** values (`Installed`, `NotPresent`).
- The requested capability `identity` is not recognized by DISM.
- The DISM API returned an error while querying or modifying capability state.
- The process is not running with elevated privileges.

## See also

- [Microsoft.Windows/OptionalFeatureList resource][06]
- [Windows features on demand documentation][07]

<!-- Link reference definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/get-feature-on-demand.md
[03]: ./examples/install-remove-feature-on-demand.md
[04]: ./examples/export-features-on-demand.md
[05]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[06]: ../OptionalFeatureList/index.md
[07]: /windows-hardware/manufacture/desktop/features-on-demand-v2--capabilities
