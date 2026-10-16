---
description: Microsoft.Windows/AuditPolicy resource reference documentation
ms.date:     10/16/2026
ms.topic:    reference
title:       Microsoft.Windows/AuditPolicy
---

# Microsoft.Windows/AuditPolicy

## Synopsis

Manage success and failure auditing for Windows advanced audit policy subcategories.

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
    type: Microsoft.Windows/AuditPolicy
    properties:
      # Key properties
      subcategory: string
      # Instance properties
      success: bool
      failure: bool
```

## Description

The `Microsoft.Windows/AuditPolicy` resource enables you to idempotently manage the advanced
audit policy of a Windows system. These are the same settings that `auditpol.exe` and the
**Advanced Audit Policy Configuration** node of the Local Security Policy console manage. Each
instance of the resource represents one audit subcategory, such as `Logon` or
`Credential Validation`, and controls whether successful and failed events in that subcategory
are written to the Security event log.

The resource can:

- Retrieve the current success and failure auditing of a subcategory.
- Enable or disable success auditing, failure auditing, or both for a subcategory. A flag that
  you omit from the desired state is left unchanged.
- Export the auditing of every subcategory on the system.

You can identify a subcategory by its English name, regardless of the system language, or by its
GUID. The resource always reports the English names, so configuration documents and exported
state are portable between systems with different display languages.

Windows defines the following categories and subcategories:

| Category           | Subcategories                                                                                                                                                                                                                                                                                                              |
|:-------------------|:---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| System             | Security State Change, Security System Extension, System Integrity, IPsec Driver, Other System Events                                                                                                                                                                                                                      |
| Logon/Logoff       | Logon, Logoff, Account Lockout, IPsec Main Mode, IPsec Quick Mode, IPsec Extended Mode, Special Logon, Other Logon/Logoff Events, Network Policy Server, User / Device Claims, Group Membership, Access Rights                                                                                                              |
| Object Access      | File System, Registry, Kernel Object, SAM, Certification Services, Application Generated, Handle Manipulation, File Share, Filtering Platform Packet Drop, Filtering Platform Connection, Other Object Access Events, Detailed File Share, Removable Storage, Central Policy Staging                                        |
| Privilege Use      | Sensitive Privilege Use, Non Sensitive Privilege Use, Other Privilege Use Events                                                                                                                                                                                                                                           |
| Detailed Tracking  | Process Creation, Process Termination, DPAPI Activity, RPC Events, Plug and Play Events, Token Right Adjusted Events                                                                                                                                                                                                       |
| Policy Change      | Audit Policy Change, Authentication Policy Change, Authorization Policy Change, MPSSVC Rule-Level Policy Change, Filtering Platform Policy Change, Other Policy Change Events                                                                                                                                               |
| Account Management | User Account Management, Computer Account Management, Security Group Management, Distribution Group Management, Application Group Management, Other Account Management Events                                                                                                                                              |
| DS Access          | Directory Service Access, Directory Service Changes, Directory Service Replication, Detailed Directory Service Replication                                                                                                                                                                                                 |
| Account Logon      | Credential Validation, Kerberos Service Ticket Operations, Other Account Logon Events, Kerberos Authentication Service                                                                                                                                                                                                     |

To list the subcategories and GUIDs that your system supports, run `auditpol /list /subcategory:* /v`
or use the **Export** operation of this resource.

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
- When the audit policy is managed by Group Policy, the domain or local Group Policy setting
  overwrites the local setting at the next policy refresh. Manage the setting through Group
  Policy instead, or exclude the subcategory from Group Policy.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of a subcategory.
- `set` - You can use the resource to enforce the desired auditing of a subcategory.
- `export` - You can use the resource to export the auditing of every subcategory on the system.
- `whatIf` - You can use the resource with the `--what-if` option of the `dsc config set` and
  `dsc resource set` commands to see which flags would change without applying them.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][01].

## Examples

1. [Get audit policy][02] - Shows how to retrieve and export the audit policy with the
   `dsc resource` commands.
1. [Configure audit policy][03] - Shows how to enforce the desired auditing of subcategories using
   a DSC configuration document.

## Properties

The following list describes the properties for the resource.

- **Key properties:** <a id="key-properties"></a> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][04].

  - [subcategory](#subcategory) - The subcategory to manage, as an English name or a GUID.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [success](#success) - Whether successful events in the subcategory are audited.
  - [failure](#failure) - Whether failed events in the subcategory are audited.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][05].

  - [guid](#guid) - The GUID of the subcategory.
  - [category](#category) - The English name of the category that contains the subcategory.

### subcategory

<details><summary>Expand for <code>subcategory</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : true
IsKey       : true
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The subcategory to manage. Specify either the English name of the subcategory, such as `Logon`
or `Credential Validation`, or its GUID, such as `{0CCE9215-69AE-11D9-BED3-505054503030}`. Names
are matched case-insensitively and the braces around a GUID are optional. The resource returns the
value exactly as you specified it. **Export** returns the English name.

If the subcategory isn't a known English name or GUID, the resource falls back to the names that
the local system reports in its display language, so localized names are also accepted.

### success

<details><summary>Expand for <code>success</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

Defines whether successful events in the subcategory are audited. Set this property to `true` to
enable success auditing or `false` to disable it. When you omit this property from a **Set**
operation, the current success auditing is left unchanged.

### failure

<details><summary>Expand for <code>failure</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

Defines whether failed events in the subcategory are audited. Set this property to `true` to
enable failure auditing or `false` to disable it. When you omit this property from a **Set**
operation, the current failure auditing is left unchanged.

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

The GUID of the subcategory, in braces and upper-case, for example
`{0CCE9215-69AE-11D9-BED3-505054503030}`. The resource returns this property and you can't set
it.

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

The English name of the audit policy category that contains the subcategory, for example
`Logon/Logoff`. The resource returns this property and you can't set it.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
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
valid instance, or because the `subcategory` isn't a known subcategory name or GUID.

### Exit code 3

Indicates the resource operation failed due to an error raised by the Windows audit policy APIs,
or the result could not be serialized.

### Exit code 4

Indicates the resource operation failed because the process isn't elevated. Reading or changing
the audit policy requires an elevated process.

## See also

- [Microsoft.Windows/AuditPolicyOption resource][06]
- [DSC resource capabilities][01]
- [DSC resource properties][07]

<!-- Link definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/get-audit-policy.md
[03]: ./examples/configure-audit-policy.md
[04]: ../../../../../concepts/resources/properties.md#key-resource-properties
[05]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[06]: ../AuditPolicyOption/index.md
[07]: ../../../../../concepts/resources/properties.md
