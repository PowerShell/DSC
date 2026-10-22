---
description: Microsoft.Windows/UserRightsAssignment resource reference documentation
ms.date:     10/19/2026
ms.topic:    reference
title:       Microsoft.Windows/UserRightsAssignment
---

# Microsoft.Windows/UserRightsAssignment

## Synopsis

Manage which accounts hold a Windows user right or privilege.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows, Security, UserRights]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/UserRightsAssignment
    properties:
      # Key properties
      right: string
      # Instance properties
      identity:
        - string
      setAction: replace | add | remove
```

## Description

The `Microsoft.Windows/UserRightsAssignment` resource enables you to idempotently manage the
accounts that hold a user right or privilege on a Windows system. These are the settings under
**Local Policies > User Rights Assignment** in the Local Security Policy console, which
`secedit` exports as the `[Privilege Rights]` section of a security template. Each instance of the
resource represents one right, such as `SeServiceLogonRight` (Log on as a service), and lists the
accounts that hold it.

The resource can:

- Retrieve the accounts that hold a right.
- Make the listed accounts the only holders of a right, grant a right to additional accounts, or
  revoke a right from specific accounts.
- Test whether a right is held by exactly, at least, or none of the listed accounts.
- Export the holders of every right on the system.

The resource reads and writes the policy through the Local Security Authority (LSA) account
rights APIs, not through `secedit`, so it needs no template files and works on any system
language. Accounts are compared by security identifier (SID), so `Administrators`,
`BUILTIN\Administrators` and `S-1-5-32-544` all refer to the same holder.

Windows defines the following user rights:

| Constant                                    | Local Security Policy name                                          |
|:--------------------------------------------|:--------------------------------------------------------------------|
| `SeTrustedCredManAccessPrivilege`           | Access Credential Manager as a trusted caller                       |
| `SeNetworkLogonRight`                       | Access this computer from the network                               |
| `SeTcbPrivilege`                            | Act as part of the operating system                                 |
| `SeMachineAccountPrivilege`                 | Add workstations to domain                                          |
| `SeIncreaseQuotaPrivilege`                  | Adjust memory quotas for a process                                  |
| `SeInteractiveLogonRight`                   | Allow log on locally                                                |
| `SeRemoteInteractiveLogonRight`             | Allow log on through Remote Desktop Services                        |
| `SeBackupPrivilege`                         | Back up files and directories                                       |
| `SeChangeNotifyPrivilege`                   | Bypass traverse checking                                            |
| `SeSystemtimePrivilege`                     | Change the system time                                              |
| `SeTimeZonePrivilege`                       | Change the time zone                                                |
| `SeCreatePagefilePrivilege`                 | Create a pagefile                                                   |
| `SeCreateTokenPrivilege`                    | Create a token object                                               |
| `SeCreateGlobalPrivilege`                   | Create global objects                                               |
| `SeCreatePermanentPrivilege`                | Create permanent shared objects                                     |
| `SeCreateSymbolicLinkPrivilege`             | Create symbolic links                                               |
| `SeDebugPrivilege`                          | Debug programs                                                      |
| `SeDenyNetworkLogonRight`                   | Deny access to this computer from the network                       |
| `SeDenyBatchLogonRight`                     | Deny log on as a batch job                                          |
| `SeDenyServiceLogonRight`                   | Deny log on as a service                                            |
| `SeDenyInteractiveLogonRight`               | Deny log on locally                                                 |
| `SeDenyRemoteInteractiveLogonRight`         | Deny log on through Remote Desktop Services                         |
| `SeEnableDelegationPrivilege`               | Enable computer and user accounts to be trusted for delegation      |
| `SeRemoteShutdownPrivilege`                 | Force shutdown from a remote system                                 |
| `SeAuditPrivilege`                          | Generate security audits                                            |
| `SeImpersonatePrivilege`                    | Impersonate a client after authentication                           |
| `SeIncreaseWorkingSetPrivilege`             | Increase a process working set                                      |
| `SeIncreaseBasePriorityPrivilege`           | Increase scheduling priority                                        |
| `SeLoadDriverPrivilege`                     | Load and unload device drivers                                      |
| `SeLockMemoryPrivilege`                     | Lock pages in memory                                                |
| `SeBatchLogonRight`                         | Log on as a batch job                                               |
| `SeServiceLogonRight`                       | Log on as a service                                                 |
| `SeSecurityPrivilege`                       | Manage auditing and security log                                    |
| `SeRelabelPrivilege`                        | Modify an object label                                              |
| `SeSystemEnvironmentPrivilege`              | Modify firmware environment values                                  |
| `SeDelegateSessionUserImpersonatePrivilege` | Obtain an impersonation token for another user in the same session  |
| `SeManageVolumePrivilege`                   | Perform volume maintenance tasks                                    |
| `SeProfileSingleProcessPrivilege`           | Profile single process                                              |
| `SeSystemProfilePrivilege`                  | Profile system performance                                          |
| `SeUndockPrivilege`                         | Remove computer from docking station                                |
| `SeAssignPrimaryTokenPrivilege`             | Replace a process level token                                       |
| `SeRestorePrivilege`                        | Restore files and directories                                       |
| `SeShutdownPrivilege`                       | Shut down the system                                                |
| `SeSyncAgentPrivilege`                      | Synchronize directory service data                                  |
| `SeTakeOwnershipPrivilege`                  | Take ownership of files or other objects                            |

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- All operations, including **Get**, **Test** and **Export**, require an elevated (administrator)
  process context because the LSA only lets administrators enumerate account rights. DSC refuses
  to invoke the resource from a non-elevated process.
- When user rights are managed by Group Policy, the domain or local Group Policy setting
  overwrites the local assignment at the next policy refresh. Manage the right through Group
  Policy instead, or exclude it from Group Policy.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the accounts that hold a right.
- `set` - You can use the resource to enforce the holders of a right.
- `test` - The resource implements its own test. It compares the current and desired holders by
  SID and honours [setAction](#setaction), so any accepted spelling of an account matches.
- `export` - You can use the resource to export the holders of every right on the system.
- `whatIf` - You can use the resource with the `--what-if` option of the `dsc config set` and
  `dsc resource set` commands to see which accounts would be granted or revoked without applying
  the change. The preview honours [setAction](#setaction), so with `add` or `remove` it keeps the
  holders that you didn't list.

For more information about resource capabilities, see [DSC resource capabilities][01].

## Examples

1. [Get user rights assignments][02] - Shows how to retrieve, test and export user rights with the
   `dsc resource` commands.
1. [Configure user rights assignments][03] - Shows how to enforce user rights using a DSC
   configuration document.

## Properties

The following list describes the properties for the resource.

- **Key properties:** <a id="key-properties"></a> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][04].

  - [right](#right) - The user right or privilege to manage.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [identity](#identity) - The accounts that hold the right.
  - [setAction](#setaction) - How the `identity` list is applied.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][05].

  - [_inDesiredState](#_indesiredstate) - Whether the current holders satisfy the desired state.

### right

<details><summary>Expand for <code>right</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : true
IsKey       : true
IsReadOnly  : false
IsWriteOnly : false
Enum        : [SeTrustedCredManAccessPrivilege, SeNetworkLogonRight, SeTcbPrivilege, SeMachineAccountPrivilege, SeIncreaseQuotaPrivilege, SeInteractiveLogonRight, SeRemoteInteractiveLogonRight, SeBackupPrivilege, SeChangeNotifyPrivilege, SeSystemtimePrivilege, SeTimeZonePrivilege, SeCreatePagefilePrivilege, SeCreateTokenPrivilege, SeCreateGlobalPrivilege, SeCreatePermanentPrivilege, SeCreateSymbolicLinkPrivilege, SeDebugPrivilege, SeDenyNetworkLogonRight, SeDenyBatchLogonRight, SeDenyServiceLogonRight, SeDenyInteractiveLogonRight, SeDenyRemoteInteractiveLogonRight, SeEnableDelegationPrivilege, SeRemoteShutdownPrivilege, SeAuditPrivilege, SeImpersonatePrivilege, SeIncreaseWorkingSetPrivilege, SeIncreaseBasePriorityPrivilege, SeLoadDriverPrivilege, SeLockMemoryPrivilege, SeBatchLogonRight, SeServiceLogonRight, SeSecurityPrivilege, SeRelabelPrivilege, SeSystemEnvironmentPrivilege, SeDelegateSessionUserImpersonatePrivilege, SeManageVolumePrivilege, SeProfileSingleProcessPrivilege, SeSystemProfilePrivilege, SeUndockPrivilege, SeAssignPrimaryTokenPrivilege, SeRestorePrivilege, SeShutdownPrivilege, SeSyncAgentPrivilege, SeTakeOwnershipPrivilege]
```

</details>

The constant of the user right or privilege to manage, such as `SeServiceLogonRight`. These are
the names that the LSA, `secedit` templates, Group Policy and security benchmarks use; the table
in the [Description](#description) maps them to the names shown in the Local Security Policy
console. The resource returns the value exactly as you specified it, and **Export** returns the
canonical casing.

### identity

<details><summary>Expand for <code>identity</code> property metadata</summary>

```yaml
Type              : array
ItemsType         : string
ItemsMustBeUnique : true
IsRequired        : false
IsKey             : false
IsReadOnly        : false
IsWriteOnly       : false
```

</details>

The accounts that hold the right. This property is required for the **Set** and **Test**
operations. Each entry must have one of the following forms:

- An account name with its domain, such as `CONTOSO\svc-sql`, `BUILTIN\Administrators`,
  `NT SERVICE\MSSQLSERVER` or `NT AUTHORITY\LOCAL SERVICE`.
- A bare account name that the local system can resolve, such as `Administrators`, or a
  well-known name that has no domain, such as `Everyone`. Well-known names are localized, so use
  the name in the language of the target system.
- A SID string, such as `S-1-5-32-544`.
- A SID string prefixed with `*`, the form that `secedit` exports, such as `*S-1-5-32-544`.

The instance schema validates each entry against these forms, so DSC rejects entries such as
`S-1-5-32-abc`, `*Administrators` or `BUILTIN\Guests\Extra` before it invokes the resource.

The resource resolves every entry to a SID. An account name that doesn't exist is an error. A SID
that doesn't resolve to an account is accepted, because Windows keeps rights assigned to deleted
accounts. Entries that resolve to the same SID count once.

On **Get** and **Export**, the resource returns each holder by its canonical name: `DOMAIN\Name`,
a bare name when the account has no domain (for example `Everyone`), or the SID string when the
SID no longer resolves to an account. An empty array means that no one holds the right.

The resource compares accounts by SID, but DSC compares the `identity` arrays as text. When an
entry isn't the canonical name of its account, such as `Administrators` or `S-1-5-32-544` for
`BUILTIN\Administrators`, the resource writes an informational message that names the canonical
form. Specify the canonical name so that the desired state matches the actual state.

### setAction

<details><summary>Expand for <code>setAction</code> property metadata</summary>

```yaml
Type         : string
IsRequired   : false
IsKey        : false
IsReadOnly   : false
IsWriteOnly  : true
DefaultValue : replace
Enum         : [replace, add, remove]
```

</details>

Defines how the **Set** operation applies the `identity` list and how the **Test** operation
judges it. The following values are valid:

| Value     | Set                                                                      | Test is in the desired state when                   |
|:----------|:-------------------------------------------------------------------------|:----------------------------------------------------|
| `replace` | The listed accounts become the only holders; other holders are revoked. | The holders are exactly the listed accounts.        |
| `add`     | The listed accounts are granted the right; other holders are kept.       | Every listed account holds the right.               |
| `remove`  | The listed accounts lose the right; other holders are kept.              | None of the listed accounts hold the right.         |

With `replace`, an empty `identity` list revokes the right from everyone. With `add` or `remove`,
an empty list changes nothing.

`remove` revokes only the direct assignment of the listed accounts. Windows grants a right to a
user when any group the user belongs to holds it, so removing a group doesn't prevent its members
from obtaining the right through another holder. To deny a right regardless of group membership,
assign the matching `SeDeny*` right instead; a deny right always takes precedence over the
matching allow right.

Because this property is write-only, DSC doesn't compare it and it never appears in the actual
state. When you use `add` or `remove`, the `differingProperties` list that DSC reports may still
name `identity` even though the resource reports the instance as in the desired state; rely on
`inDesiredState` for the verdict.

### _inDesiredState

<details><summary>Expand for <code>_inDesiredState</code> property metadata</summary>

```yaml
Type        : [boolean, 'null']
IsRequired  : false
IsKey       : false
IsReadOnly  : true
IsWriteOnly : false
```

</details>

This is the canonical `_inDesiredState` property. Returned by the **Test** operation only. Indicates whether the current holders of the right
satisfy `identity` under the chosen `setAction`.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource. The
`enum` for `right` is abbreviated; it contains the 45 constants listed in the
[Description](#description). Annotations such as `title`, `description`, `markdownDescription`
and `markdownEnumDescriptions` are omitted, and the canonical `_inDesiredState` definition is
bundled under `$defs` in the manifest.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["right"],
  "additionalProperties": false,
  "properties": {
    "right": {
      "type": "string",
      "x-dsc-key": true,
      "enum": ["SeTrustedCredManAccessPrivilege", "SeNetworkLogonRight", "..."]
    },
    "identity": {
      "type": "array",
      "uniqueItems": true,
      "items": {
        "type": "string",
        "oneOf": [
          { "pattern": "^[^\\\\*]+\\\\[^\\\\*]+$" },
          { "pattern": "^[^\\\\*]+$", "not": { "pattern": "^[Ss]-1-[0-9]" } },
          { "pattern": "^[Ss]-1-([0-9]+|0[Xx][0-9A-Fa-f]+)(-[0-9]+)*$" },
          { "pattern": "^\\*[Ss]-1-([0-9]+|0[Xx][0-9A-Fa-f]+)(-[0-9]+)*$" }
        ]
      }
    },
    "setAction": {
      "type": "string",
      "writeOnly": true,
      "default": "replace",
      "enum": ["replace", "add", "remove"]
    },
    "_inDesiredState": {
      "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/resource/properties/inDesiredState.json"
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
- [3](#exit-code-3) - Security policy error
- [4](#exit-code-4) - Elevation required

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because required arguments were missing or the operation
name was not recognized.

### Exit code 2

Indicates the resource operation failed because the JSON input could not be deserialized into a
valid instance, the `right` isn't a known user right, an entry in `identity` isn't a valid SID or
an existing account (including `*` followed by anything other than a SID string), or `identity`
was omitted from a **Set** or **Test** operation.

### Exit code 3

Indicates the resource operation failed due to an error raised by the Local Security Authority or
another Windows API, or the result could not be serialized.

### Exit code 4

Indicates the resource operation failed because the process isn't elevated. Reading or changing
user rights assignments requires an elevated process.

## See also

- [Microsoft.Windows/Registry resource][06]
- [DSC resource capabilities][01]
- [DSC resource properties][07]

<!-- Link definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/get-user-rights-assignment.md
[03]: ./examples/configure-user-rights-assignment.md
[04]: ../../../../../concepts/resources/properties.md#key-resource-properties
[05]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[06]: ../Registry/index.md
[07]: ../../../../../concepts/resources/properties.md
