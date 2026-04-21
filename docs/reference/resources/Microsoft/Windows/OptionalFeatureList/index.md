---
description: Microsoft.Windows/OptionalFeatureList resource reference documentation
ms.date:     04/21/2026
ms.topic:    reference
title:       Microsoft.Windows/OptionalFeatureList
---

# Microsoft.Windows/OptionalFeatureList

## Synopsis

Manage Windows Optional features using the DISM API.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows, dism, optionalfeature, feature]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/OptionalFeatureList
    properties:
      # Required properties
      features:
        - featureName: string
          # Instance properties
          state: Installed | NotPresent | Removed
```

## Description

The `Microsoft.Windows/OptionalFeatureList` resource enables you to idempotently manage Windows
Optional features using the DISM API. Optional features are components built into Windows that can
be enabled or disabled without downloading additional content. Examples include Hyper-V,
Windows Subsystem for Linux, and Internet Information Services (IIS).

The resource can:

- Retrieve the current state of one or more optional features by name.
- Enable optional features (`Installed`).
- Disable optional features while keeping the feature payload staged (`NotPresent`).
- Disable optional features and remove the associated payload from the system (`Removed`).
- Export a list of all optional features on the system, optionally filtered by name, state, display
  name, or description.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- All operations require an elevated (administrator) process context.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of one or more optional feature
  instances.
- `set` - You can use the resource to enforce the desired state for one or more optional feature
  instances.
- `export` - You can use the resource to enumerate all optional features on the system, with
  optional filtering.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][01].

## Examples

1. [Get optional feature state][02] - Shows how to retrieve the current state of a Windows
   Optional Feature.
1. [Enable and disable optional features][03] - Shows how to enable and disable Windows Optional
   Features using the `dsc resource set` command.
1. [Export optional features][04] - Shows how to enumerate all optional features on the system,
   with and without filters.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource.

  - [features](#features) - An array of optional feature entries.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][05].

  - [_restartRequired](#_restartrequired) - Indicates that a system restart is required to complete
    the state change.

### features

<details><summary>Expand for <code>features</code> property metadata</summary>

```yaml
Type       : array
IsRequired : true
IsKey      : false
IsReadOnly : false
```

</details>

An array of optional feature entries. Each entry is an object describing a Windows Optional Feature.
For the **Get** operation, each entry must specify [`featureName`](#featurename). For the **Set**
operation, each entry must specify both [`featureName`](#featurename) and [`state`](#state). For
the **Export** operation, the array is optional and each entry can filter results using
[`featureName`](#featurename), [`state`](#state), [`displayName`](#displayname), or
[`description`](#description) with wildcard support.

Each entry in `features` has the following properties:

- [featureName](#featurename) - The name of the optional feature.
- [_exist](#_exist) - Indicates whether the feature is recognized by DISM.
- [state](#state) - The current or desired state of the feature.
- [displayName](#displayname) - The display name of the feature.
- [description](#description) - The description of the feature.
- [restartRequired](#restartrequired) - Whether a restart is required after a state change.

#### featureName

<details><summary>Expand for <code>features[*].featureName</code> property metadata</summary>

```yaml
Type       : string
IsRequired : true (get, set) / false (export)
IsKey      : false
IsReadOnly : false
```

</details>

The name of the Windows Optional Feature. For **Get** and **Set** operations, this property is
required for each entry. For the **Export** operation, it's optional and supports wildcard (`*`)
patterns for case-insensitive filtering.

Use the `dism /Online /Get-Features` command to list available feature names on your system.

#### _exist

<details><summary>Expand for <code>features[*]._exist</code> property metadata</summary>

```yaml
Type       : boolean
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

Indicates whether the feature exists on the system. The resource sets this property to `false` in
the **Get** response when the requested `featureName` is not recognized by DISM. When `_exist` is
`false`, the `state`, `displayName`, `description`, and `restartRequired` properties are not
returned.

#### state

<details><summary>Expand for <code>features[*].state</code> property metadata</summary>

```yaml
Type         : string
IsRequired   : true (set) / false (get, export)
IsKey        : false
IsReadOnly   : false (set input) / true (get/export output)
ValidValues  : [NotPresent, UninstallPending, Staged, Removed, Installed,
                InstallPending, Superseded, PartiallyInstalled]
SetValues    : [Installed, NotPresent, Removed]
```

</details>

The state of the optional feature. **Get** and **Export** operations return one of the eight DISM
feature state values. **Set** operations accept only the following three values as desired state:

| Value        | Description                                                               |
|:-------------|:--------------------------------------------------------------------------|
| `Installed`  | The feature is enabled. The resource enables the feature if not already.  |
| `NotPresent` | The feature is disabled but the payload remains on disk (staged).         |
| `Removed`    | The feature is disabled and the payload is removed from the system.       |

The following table describes all possible state values returned by **Get** and **Export**:

| Value                | Description                                                         |
|:---------------------|:--------------------------------------------------------------------|
| `NotPresent`         | The feature is disabled with its payload removed or never staged.   |
| `UninstallPending`   | A disable operation is pending, requiring a restart to complete.    |
| `Staged`             | The feature payload is on disk but the feature is not enabled.      |
| `Removed`            | The feature is disabled and its source payload has been removed.    |
| `Installed`          | The feature is enabled and fully operational.                       |
| `InstallPending`     | An enable operation is pending, requiring a restart to complete.    |
| `Superseded`         | The feature has been replaced by another component.                 |
| `PartiallyInstalled` | The feature is only partially installed.                            |

#### displayName

<details><summary>Expand for <code>features[*].displayName</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

The human-readable display name of the optional feature. This property is returned by **Get** and
**Export** operations. For **Export** operations, you can specify this property as a filter value
with wildcard (`*`) support for case-insensitive matching.

#### description

<details><summary>Expand for <code>features[*].description</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : true
```

</details>

A brief description of the optional feature. This property is returned by **Get** and **Export**
operations. For **Export** operations, you can specify this property as a filter value with
wildcard (`*`) support for case-insensitive matching.

#### restartRequired

<details><summary>Expand for <code>features[*].restartRequired</code> property metadata</summary>

```yaml
Type         : string
IsRequired   : false
IsKey        : false
IsReadOnly   : true
ValidValues  : [No, Possible, Required]
```

</details>

Indicates whether a system restart is required after enabling or disabling the feature. This
property is returned by **Get** and **Export** operations and cannot be set.

| Value      | Description                                               |
|:-----------|:----------------------------------------------------------|
| `No`       | No restart is required after the state change.            |
| `Possible` | A restart may be required depending on system conditions. |
| `Required` | A restart is required to complete the state change.       |

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
  "required": ["features"],
  "additionalProperties": false,
  "properties": {
    "_restartRequired": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": true
      }
    },
    "features": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "featureName": { "type": "string" },
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
          "restartRequired": {
            "type": "string",
            "enum": ["No", "Possible", "Required"]
          }
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

- The `features` array is empty.
- The `featureName` property is missing from a feature entry in a **Get** or **Set** operation.
- The `state` property is missing from a feature entry in a **Set** operation.
- The desired `state` value is not one of the accepted **Set** values (`Installed`, `NotPresent`,
  `Removed`).
- The DISM API returned an error while querying or modifying feature state.
- The process is not running with elevated privileges.

## See also

- [Microsoft.Windows/FeatureOnDemandList resource][06]
- [Windows Optional Features documentation][07]

<!-- Link reference definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/get-optional-feature.md
[03]: ./examples/enable-disable-optional-features.md
[04]: ./examples/export-optional-features.md
[05]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[06]: ../FeatureOnDemandList/index.md
[07]: /windows-server/administration/windows-commands/dism/dism-operating-system-package-servicing-command-line-options
