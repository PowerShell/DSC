---
description: Microsoft.Windows/UpdateList resource reference documentation
ms.date:     09/25/2026
ms.topic:    reference
title:       Microsoft.Windows/UpdateList
---

# Microsoft.Windows/UpdateList

## Synopsis

Query and install Windows Updates by using the Windows Update Agent APIs.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [windows, update, patch, security]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/UpdateList
    properties:
      # Required properties
      updates:
        - title: <exact update title>
          # Or use another supported query criterion
          id: <update GUID>
          isInstalled: true | false
          kbArticleIds:
            - <KB article ID>
          updateType: Software | Driver
          msrcSeverity: Critical | Important | Moderate | Low
```

## Description

The `Microsoft.Windows/UpdateList` resource uses the Windows Update Agent COM APIs to retrieve
information about updates available on or installed on a Windows system. It can query individual
updates, install matching updates, and export update information with optional filters.

For **Get** and **Set** operations, each update entry must specify at least one query criterion.
All criteria in one entry must match the same update. For **Export**, the resource accepts an empty
input to return all updates, or one or more filter entries. Criteria in a filter are combined with
AND; separate filter entries are combined with OR. Title and description filters support `*`
wildcards and case-insensitive matching.

The **Set** operation downloads and installs matching updates that aren't already installed. It
requires an elevated process context. The operation can report a pending restart after installation.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- The Windows Update Agent and Windows Update service must be available.
- The **Set** operation requires an elevated (administrator) process context.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve information about updates matching one or more
  criteria.
- `set` - You can use the resource to download and install matching updates.
- `test` - You can use the resource to test whether updates are in the desired state.
- `export` - You can use the resource to enumerate updates, optionally using filters.

This resource uses the synthetic test functionality of DSC to determine whether updates are in the
desired state. The resource doesn't implement a direct delete operation.

For more information about resource capabilities, see [DSC resource capabilities][01].

## Examples

1. [Get information about an update][02] - Shows how to query an update by title or another
   criterion.
1. [Install a matching update][03] - Shows how to install an update.
1. [Export and filter updates][04] - Shows how to export all updates or filter them by status,
   title, severity, or type.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource.

  - [updates](#updates) - An array of update query, filter, or result objects.

- **Key properties:** <a id="key-properties"></a> This resource doesn't have any key properties.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional
  within an update entry. They define query criteria or the desired state for an update.

  - [updates](#updates) - The update entries to query, install, or filter.

### updates

<details><summary>Expand for <code>updates</code> property metadata</summary>

```yaml
Type       : array
IsRequired : true
IsKey      : false
IsReadOnly : false
```

</details>

An array of update entries. For **Get** and **Set**, each entry must specify at least one of
`title`, `id`, `isInstalled`, `kbArticleIds`, `updateType`, or `msrcSeverity`. For **Export**, the
array can be empty. Each filter can specify any supported filter property, and an empty filter
matches all updates. Separate filters are combined with OR. The `updates` property itself is
always required in a resource instance.

Each entry can contain the following properties:

- [title](#title) - The exact title for **Get** and **Set**, or a wildcard filter for **Export**.
- [id](#id) - The update GUID.
- [isInstalled](#isinstalled) - Whether the update is installed.
- [description](#description) - The update description or an **Export** wildcard filter.
- [isUninstallable](#isuninstallable) - Whether the update can be uninstalled.
- [kbArticleIds](#kbarticleids) - Associated KB article identifiers.
- [recommendedHardDiskSpace](#recommendedharddiskspace) - Required free disk space in MB.
- [msrcSeverity](#msrcseverity) - The MSRC severity rating.
- [securityBulletinIds](#securitybulletinids) - Associated security bulletin identifiers.
- [updateType](#updatetype) - `Software` or `Driver`.
- [installationBehavior](#installationbehavior) - Expected reboot behavior.

### title

<details><summary>Expand for <code>updates[*].title</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

The exact case-insensitive update title to match for **Get** or **Set**. For **Export**, the value
can contain `*` wildcards.

### id

<details><summary>Expand for <code>updates[*].id</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

The case-insensitive GUID that uniquely identifies the update. **Get** and **Set** require at least
one query criterion, such as `id`.

### isInstalled

<details><summary>Expand for <code>updates[*].isInstalled</code> property metadata</summary>

```yaml
Type       : boolean
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

Indicates whether the update is installed. It can be used as a query criterion for **Get** and
**Set**, or as a filter for **Export**. Update results also return this property.

### description

<details><summary>Expand for <code>updates[*].description</code> property metadata</summary>

```yaml
Type       : string
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

The detailed update description. For **Export**, the value supports case-insensitive `*` wildcard
matching.

### isUninstallable

<details><summary>Expand for <code>updates[*].isUninstallable</code> property metadata</summary>

```yaml
Type       : boolean
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

Indicates whether the update can be uninstalled. This property is available as an **Export**
filter and in returned update information.

### kbArticleIds

<details><summary>Expand for <code>updates[*].kbArticleIds</code> property metadata</summary>

```yaml
Type       : array
ItemsType  : string
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

The KB article identifiers associated with the update. For **Get** and **Set**, all specified IDs
must be present. For **Export**, any specified ID can match.

### recommendedHardDiskSpace

<details><summary>Expand for <code>updates[*].recommendedHardDiskSpace</code> property metadata</summary>

```yaml
Type       : integer
Format     : int64
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

The recommended free hard disk space in megabytes (MB) before installing the update. For **Export**,
updates with a value greater than or equal to the filter value are returned.

### msrcSeverity

<details><summary>Expand for <code>updates[*].msrcSeverity</code> property metadata</summary>

```yaml
Type        : string
ValidValues : [Critical, Important, Moderate, Low]
IsRequired  : false
IsKey       : false
IsReadOnly  : false
```

</details>

The Microsoft Security Response Center severity rating for the update.

### securityBulletinIds

<details><summary>Expand for <code>updates[*].securityBulletinIds</code> property metadata</summary>

```yaml
Type       : array
ItemsType  : string
IsRequired : false
IsKey      : false
IsReadOnly : false
```

</details>

The security bulletin identifiers associated with the update. For **Export**, any specified bulletin
identifier can match.

### updateType

<details><summary>Expand for <code>updates[*].updateType</code> property metadata</summary>

```yaml
Type        : string
ValidValues : [Software, Driver]
IsRequired  : false
IsKey       : false
IsReadOnly  : false
```

</details>

The type of the update.

### installationBehavior

<details><summary>Expand for <code>updates[*].installationBehavior</code> property metadata</summary>

```yaml
Type        : string
ValidValues : [NeverReboots, AlwaysRequiresReboot, CanRequestReboot]
IsRequired  : false
IsKey       : false
IsReadOnly  : false
```

</details>

The reboot behavior expected when the update is installed. This property is returned in update
information and isn't used as a query criterion.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource. The
validating schema only includes schema keywords that affect how the instance is validated.

```json
{
  "type": "object",
  "required": ["updates"],
  "additionalProperties": false,
  "properties": {
    "updates": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "title": { "type": "string" },
          "id": { "type": "string" },
          "isInstalled": { "type": "boolean" },
          "description": { "type": "string" },
          "isUninstallable": { "type": "boolean" },
          "kbArticleIds": { "type": "array", "items": { "type": "string" } },
          "recommendedHardDiskSpace": { "type": "integer", "format": "int64" },
          "msrcSeverity": {
            "type": "string",
            "enum": ["Critical", "Important", "Moderate", "Low"]
          },
          "securityBulletinIds": { "type": "array", "items": { "type": "string" } },
          "updateType": { "type": "string", "enum": ["Software", "Driver"] },
          "installationBehavior": {
            "type": "string",
            "enum": ["NeverReboots", "AlwaysRequiresReboot", "CanRequestReboot"]
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
- [1](#exit-code-1) - Operation failed

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the operation failed. Common causes include invalid JSON input, an empty `updates` array
for **Get** or **Set**, no matching update, an unavailable Windows Update service, COM errors, or
an update download or installation failure. The resource writes an error message to stderr.

## See also

- [DSC resource capabilities][01]
- [DSC resource properties][05]
- [Windows Update Agent API][06]

<!-- Link definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/get-update.md
[03]: ./examples/install-update.md
[04]: ./examples/export-updates.md
[05]: ../../../../../concepts/resources/properties.md
[06]: /windows/win32/wua-sdk/windows-update-agent-object-model
