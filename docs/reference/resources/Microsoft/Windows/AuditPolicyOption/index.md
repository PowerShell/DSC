---
description: Microsoft.Windows/AuditPolicyOption resource reference documentation
ms.date:     10/19/2026
ms.topic:    reference
title:       Microsoft.Windows/AuditPolicyOption
---

# Microsoft.Windows/AuditPolicyOption

## Synopsis

Manage Windows audit policy options stored under the LSA registry key.

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
    type: Microsoft.Windows/AuditPolicyOption
    properties:
      # Key properties
      name: string
      # Instance properties
      enabled: bool
```

## Description

The `Microsoft.Windows/AuditPolicyOption` resource enables you to idempotently manage the four
audit policy options that `auditpol.exe` exposes through its `/option` parameter. Windows stores
these options as values under the `HKLM\SYSTEM\CurrentControlSet\Control\Lsa` registry key, which
is also where the corresponding **Security Options** of the Local Security Policy console and
`secedit` security templates store them.

| Option                  | Security Options policy                                                     | Registry value type |
|:------------------------|:----------------------------------------------------------------------------|:--------------------|
| `CrashOnAuditFail`      | Audit: Shut down system immediately if unable to log security audits        | `REG_DWORD`         |
| `FullPrivilegeAuditing` | Audit: Audit the use of Backup and Restore privilege                        | `REG_BINARY`        |
| `AuditBaseObjects`      | Audit: Audit the access of global system objects                            | `REG_DWORD`         |
| `AuditBaseDirectories`  | Audit the access of global system directories (no Security Options policy)  | `REG_DWORD`         |

The resource reports an option as enabled when its registry value is present and non-zero, and as
disabled when the value is absent or zero. The value may be stored as `REG_DWORD`, `REG_QWORD`, or
`REG_BINARY`. When the resource sets an option, it preserves the existing registry value type and
only uses the type from the table when the value doesn't exist yet.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- **Set** operations require an elevated (administrator) process context.
- The Local Security Authority reads these values when the system starts. A change made with the
  **Set** operation takes effect after the next restart. The operation reports this in the
  read-only [_restartRequired](#_restartrequired) property, which DSC aggregates for the
  [restartRequired()][07] configuration function, and the resource also writes a warning to say so.

> [!WARNING]
> Enabling `CrashOnAuditFail` causes Windows to halt with a `STOP` error whenever it can't write
> an audit record, for example when the Security event log is full. After such a halt, Windows
> sets the registry value to `2` and only administrators can sign in until the value is reset.
> The resource reports a value of `2` as `enabled: true`.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an option.
- `set` - You can use the resource to enable or disable an option.
- `export` - You can use the resource to export the state of all four options.
- `whatIf` - You can use the resource with the `--what-if` option of the `dsc config set` and
  `dsc resource set` commands to see whether an option would change without applying it.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][01].

## Examples

1. [Configure audit policy options][02] - Shows how to enforce the desired state of audit policy
   options using a DSC configuration document.

## Properties

The following list describes the properties for the resource.

- **Key properties:** <a id="key-properties"></a> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][03].

  - [name](#name) - The option to manage.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [enabled](#enabled) - Whether the option is enabled.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][06].

  - [_restartRequired](#_restartrequired) - Indicates that the system must restart before a change
    takes effect.

### name

<details><summary>Expand for <code>name</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : true
IsKey       : true
IsReadOnly  : false
IsWriteOnly : false
Enum        : [CrashOnAuditFail, FullPrivilegeAuditing, AuditBaseObjects, AuditBaseDirectories]
```

</details>

The audit policy option to manage. The value is the name of the registry value that stores the
option and matches the name that `auditpol /get /option:<name>` uses.

### enabled

<details><summary>Expand for <code>enabled</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

Defines whether the option is enabled. This property is optional for the **Get** operation and
required for the **Set** operation. A change takes effect after the next restart.

### _restartRequired

<details><summary>Expand for <code>_restartRequired</code> property metadata</summary>

```yaml
Type        : array
IsRequired  : false
IsKey       : false
IsReadOnly  : true
IsWriteOnly : false
```

</details>

Returned by the **Set** operation after it changed the option. Each entry is an object whose
`system` property names the computer that must restart before the Local Security Authority reads
the new value. DSC adds the entries to the `restartRequired` execution information of the
operation, so a configuration document can query `[restartRequired('system')]` after this resource
has run. The property is omitted when the option was already in the desired state and in what-if
mode.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["name"],
  "additionalProperties": false,
  "properties": {
    "name": {
      "type": "string",
      "x-dsc-key": true,
      "enum": [
        "CrashOnAuditFail",
        "FullPrivilegeAuditing",
        "AuditBaseObjects",
        "AuditBaseDirectories"
      ]
    },
    "enabled": {
      "type": "boolean"
    },
    "_metadata": {
      "type": "object",
      "readOnly": true,
      "additionalProperties": false,
      "properties": {
        "whatIf": {
          "type": "array",
          "items": {
            "type": "string"
          }
        }
      }
    },
    "_restartRequired": {
      "type": "array",
      "readOnly": true,
      "items": {
        "type": "object",
        "required": ["system"],
        "additionalProperties": false,
        "properties": {
          "system": {
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
valid instance, or because `enabled` was omitted from a **Set** operation.

### Exit code 3

Indicates the resource operation failed because the registry value could not be read or written,
has an unexpected type, or the result could not be serialized.

### Exit code 4

Indicates the resource operation failed because the process isn't elevated. Changing an audit
policy option requires an elevated process.

## See also

- [Microsoft.Windows/AuditPolicy resource][04]
- [Microsoft.Windows/Registry resource][05]
- [DSC resource capabilities][01]
- [DSC resource properties][06]

<!-- Link definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/configure-audit-policy-option.md
[03]: ../../../../../concepts/resources/properties.md#key-resource-properties
[04]: ../AuditPolicy/index.md
[05]: ../Registry/index.md
[06]: ../../../../../concepts/resources/properties.md
[07]: ../../../../schemas/config/functions/restartRequired.md
