---
description: Microsoft.Windows/AuditPolicyList resource reference documentation
ms.date:     10/19/2026
ms.topic:    reference
title:       Microsoft.Windows/AuditPolicyList
---

# Microsoft.Windows/AuditPolicyList

## Synopsis

Manage success and failure auditing for several Windows advanced audit policy subcategories at
once.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows, Security, Audit]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/AuditPolicyList
    properties:
      # Required properties
      policies:
        - subcategory: string
          success: bool
          failure: bool
```

## Description

The `Microsoft.Windows/AuditPolicyList` resource enables you to idempotently manage the advanced
audit policy of many subcategories with a single instance. Each entry in the `policies` list has
the same properties as an instance of the [Microsoft.Windows/AuditPolicy][01] resource and
behaves the same way:

- You identify the subcategory by its English name or its GUID, and the resource returns the
  value exactly as you specified it.
- A `success` or `failure` flag that you omit from an entry is left unchanged.
- The resource returns the read-only `guid` and `category` properties for every entry.

Use this resource for an audit baseline that configures many subcategories, such as the
recommendations of a security benchmark. The resource validates every entry before it changes
anything, and it applies all the changes with a single call to the audit policy API. When any
entry is invalid, such as an unknown subcategory or a subcategory listed twice, the resource
changes nothing.

For the list of categories and subcategories, see the [AuditPolicy][01] resource.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- All operations, including **Get** and **Export**, require an elevated (administrator) process
  context because reading the audit policy requires the `SeSecurityPrivilege` privilege. DSC
  refuses to invoke the resource from a non-elevated process.
- The account running the elevated process must hold the **Manage auditing and security log**
  user right. When policy removes that right, the elevated process still lacks
  `SeSecurityPrivilege` and the resource fails with [exit code 3](#exit-code-3) and an error
  that names the missing privilege.
- When the audit policy is managed by Group Policy, the domain or local Group Policy setting
  overwrites the local setting at the next policy refresh. Manage the setting through Group
  Policy instead, or exclude the subcategories from Group Policy.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of every listed subcategory.
- `set` - You can use the resource to enforce the desired auditing of every listed subcategory.
- `export` - You can use the resource to export the auditing of every subcategory on the system
  as a single instance.
- `whatIf` - You can use the resource with the `--what-if` option of the `dsc config set` and
  `dsc resource set` commands to see which flags would change without applying them.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. An instance is in the desired state when every listed subcategory has the flags
that its entry specifies. For more information about resource capabilities, see
[DSC resource capabilities][02].

## Examples

1. [Get audit policy list][03] - Shows how to retrieve, test and export several subcategories with
   the `dsc resource` commands.
1. [Configure an audit policy baseline][04] - Shows how to enforce the auditing of several
   subcategories with one instance in a DSC configuration document.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource. An instance that doesn't define each of
  these properties is invalid. For more information, see the "Required resource properties"
  section in [DSC resource properties][05].

  - [policies](#policies) - The subcategories to manage.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][06].

  - [guid](#guid) - The GUID of a listed subcategory.
  - [category](#category) - The English name of the category that contains a listed subcategory.

### policies

<details><summary>Expand for <code>policies</code> property metadata</summary>

```yaml
Type       : array
ItemsType  : object
MinItems   : 1
IsRequired : true
IsKey      : false
IsReadOnly : false
```

</details>

The subcategories to manage. Each entry is an object with the following properties:

- `subcategory` (string, required) - The subcategory to manage. Specify either the English name of
  the subcategory, such as `Logon`, or its GUID, such as
  `{0CCE9215-69AE-11D9-BED3-505054503030}`. Names are matched case-insensitively and the braces
  around a GUID are optional. The resource returns the value exactly as you specified it.
- `success` (boolean, optional) - Whether successful events in the subcategory are audited. When
  you omit this property from a **Set** operation, the current success auditing is left
  unchanged.
- `failure` (boolean, optional) - Whether failed events in the subcategory are audited. When you
  omit this property from a **Set** operation, the current failure auditing is left unchanged.

List each subcategory once. The resource rejects a list that names the same subcategory more than
once, even when the entries use different spellings or a name and a GUID, because the entries
would compete for the same flags. **Get** and **Set** return the entries in the order you listed
them. **Export** returns every subcategory on the system, grouped by category.

### guid

<details><summary>Expand for <code>guid</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : true
IsWriteOnly : false
```

</details>

Returned for every entry in `policies`: the GUID of the subcategory, in braces and upper-case,
for example `{0CCE9215-69AE-11D9-BED3-505054503030}`.

### category

<details><summary>Expand for <code>category</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : true
IsWriteOnly : false
```

</details>

Returned for every entry in `policies`: the English name of the audit policy category that
contains the subcategory, for example `Logon/Logoff`.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["policies"],
  "additionalProperties": false,
  "properties": {
    "policies": {
      "type": "array",
      "minItems": 1,
      "items": {
        "$ref": "#/$defs/policy"
      }
    },
    "_metadata": {
      "type": "object",
      "readOnly": true,
      "properties": {
        "whatIf": {
          "type": "array",
          "items": {
            "type": "string"
          }
        }
      }
    }
  },
  "$defs": {
    "policy": {
      "type": "object",
      "required": ["subcategory"],
      "additionalProperties": false,
      "properties": {
        "subcategory": {
          "type": "string",
          "minLength": 1
        },
        "success": {
          "type": "boolean"
        },
        "failure": {
          "type": "boolean"
        },
        "guid": {
          "type": "string",
          "readOnly": true
        },
        "category": {
          "type": "string",
          "readOnly": true
        }
      }
    }
  }
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Invalid arguments
- [2](#exit-code-2) - Invalid input
- [3](#exit-code-3) - Audit policy error
- [4](#exit-code-4) - Elevation required

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because required arguments were missing or the operation
name was not recognized.

### Exit code 2

Indicates the resource operation failed because the JSON input could not be deserialized into a
valid instance, an entry names a subcategory that isn't a known subcategory name or GUID, or two
entries name the same subcategory.

### Exit code 3

Indicates the resource operation failed due to an error raised by the Windows audit policy APIs,
or the result could not be serialized.

### Exit code 4

Indicates the resource operation failed because the process isn't elevated. Reading or changing
the audit policy requires an elevated process.

## See also

- [Microsoft.Windows/AuditPolicy resource][01]
- [Microsoft.Windows/AuditPolicyOption resource][07]
- [DSC resource capabilities][02]
- [DSC resource properties][08]

<!-- Link definitions -->
[01]: ../AuditPolicy/index.md
[02]: ../../../../../concepts/resources/capabilities.md
[03]: ./examples/get-audit-policy-list.md
[04]: ./examples/configure-audit-policy-list.md
[05]: ../../../../../concepts/resources/properties.md#required-resource-properties
[06]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[07]: ../AuditPolicyOption/index.md
[08]: ../../../../../concepts/resources/properties.md
