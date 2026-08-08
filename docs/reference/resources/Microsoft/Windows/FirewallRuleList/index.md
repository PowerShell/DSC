---
description: Microsoft.Windows/FirewallRuleList resource reference documentation
ms.date:     05/09/2026
ms.topic:    reference
title:       Microsoft.Windows/FirewallRuleList
---

# Microsoft.Windows/FirewallRuleList

## Synopsis

Manage Windows Firewall rules using the netfw.h APIs.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows, Firewall]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/FirewallRuleList
    properties:
      rules:
        - name: string
          # Rule properties
          action:
          applicationName:
          description:
          direction:
          edgeTraversal:
          enabled:
          grouping:
          interfaceTypes:
          localAddresses:
          localPorts:
          profiles:
          protocol:
          remoteAddresses:
          remotePorts:
          serviceName:
          _exist:
```

## Description

The `Microsoft.Windows/FirewallRuleList` resource enables you to idempotently manage Windows
Firewall rules through the `netfw.h` COM APIs. A single instance of the resource manages an array
of rules, allowing you to create, update, or remove multiple rules in one operation.

The resource can:

- Retrieve the full configuration of one or more named firewall rules.
- Create rules that don't exist, update properties of rules that do, and remove rules by setting
  `_exist: false`.
- Export all registered firewall rules, with optional AND/OR filtering by rule properties.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

> [!IMPORTANT]
> The `_exist` property on a rule item behaves differently from most DSC resources. When a rule
> exists in the Windows Firewall store, `_exist` is **omitted** from the returned state (absent
> means present). When a rule is not found, `_exist: false` appears in the response. This means
> that a missing `_exist` field in the actual state always indicates the rule exists.

## Requirements

- The resource is only usable on Windows systems.
- **Set** and **Export** operations require an elevated (administrator) process context. Invoking
  the resource for these operations in a non-elevated process context causes the resource to raise
  an error.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of one or more firewall rules.
- `set` - You can use the resource to enforce the desired state of one or more firewall rules,
  including creating and removing rules.
- `export` - You can use the resource to export all firewall rules registered on the system,
  with optional filtering.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][01].

## Examples

1. [Get firewall rule state][03] - Shows how to retrieve the current state of a Windows Firewall
   rule and toggle it with the `dsc resource` commands.
1. [Configure firewall rules][04] - Shows how to create and manage multiple Windows Firewall rules
   using a DSC configuration document.

## Properties

The `Microsoft.Windows/FirewallRuleList` instance has one required property at the root level.

- **Required properties:** <a id="required-properties"></a>

  - [rules](#rules) - An array of firewall rule objects to get, set, or use as export filters.

### rules

<details><summary>Expand for <code>rules</code> property metadata</summary>

```yaml
Type        : array
IsRequired  : true
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

An array of firewall rule objects. For **Get** and **Set** operations, each entry in the array
must include a [name](#name) property that identifies the rule. For **Export**, each entry acts
as a filter — all properties within a single entry are ANDed together, and multiple entries are
ORed. The array must contain at least one entry for **Get** and **Set** operations.

Each entry in the `rules` array supports the following properties.

### name

<details><summary>Expand for <code>name</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : true for get and set
IsKey       : true (within the rules array)
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The Windows Firewall rule name as registered in the firewall store. This is the exact name shown
in the Windows Firewall console. Name matching is case-insensitive. Wildcard patterns using `*`
are supported for **Export** filter entries.

### _exist

<details><summary>Expand for <code>_exist</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : false (writable for set to remove a rule)
IsWriteOnly : false
```

</details>

Indicates whether a firewall rule exists. The behavior of this property differs from most DSC
resources:

- When a rule _exists_, `_exist` is _omitted_ from the returned state. Absence means the rule
  is present.
- When a rule is _not found_, `_exist: false` appears in the response.
- In a **Set** operation, set `_exist: false` on a rule entry to remove the rule if it exists.

### description

<details><summary>Expand for <code>description</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

A human-readable description of the firewall rule shown in the Windows Firewall console.

### applicationName

<details><summary>Expand for <code>applicationName</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The fully qualified path to the application executable associated with the rule — for example,
`C:\Program Files\MyApp\myapp.exe`. When specified, the rule only applies to traffic from or
to that application.

### serviceName

<details><summary>Expand for <code>serviceName</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The Windows service short name associated with the rule. When specified, the rule only applies
to traffic from or to that service.

### protocol

<details><summary>Expand for <code>protocol</code> property metadata</summary>

```yaml
Type                  : integer
IsRequired            : false
IsKey                 : false
IsReadOnly            : false
IsWriteOnly           : false
InclusiveMinimumValue : 0
InclusiveMaximumValue : 256
```

</details>

The IANA IP protocol number for the rule. The following values are commonly used:

| Value | Protocol            |
|------:|:--------------------|
|   `1` | ICMPv4              |
|   `6` | TCP                 |
|  `17` | UDP                 |
|  `58` | ICMPv6              |
| `256` | Any (all protocols) |

### localPorts

<details><summary>Expand for <code>localPorts</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

A comma-separated list of local port numbers or ranges for the rule — for example, `80,443` or
`8000-8080`. Only valid when `protocol` is `6` (TCP) or `17` (UDP).

### remotePorts

<details><summary>Expand for <code>remotePorts</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

A comma-separated list of remote port numbers or ranges for the rule. Only valid when `protocol`
is `6` (TCP) or `17` (UDP).

### localAddresses

<details><summary>Expand for <code>localAddresses</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

A comma-separated list of local IP addresses or subnets in CIDR notation for the rule — for
example, `192.168.1.0/24,10.0.0.1`.

### remoteAddresses

<details><summary>Expand for <code>remoteAddresses</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

A comma-separated list of remote IP addresses or subnets in CIDR notation for the rule.

### direction

<details><summary>Expand for <code>direction</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
Enum        : [Inbound, Outbound]
```

</details>

The direction of network traffic the rule applies to.

| Value      | Description                           |
|:-----------|:--------------------------------------|
| `Inbound`  | The rule applies to incoming traffic. |
| `Outbound` | The rule applies to outgoing traffic. |

### action

<details><summary>Expand for <code>action</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
Enum        : [Allow, Block]
```

</details>

The action taken by the rule when traffic matches.

| Value   | Description                                 |
|:--------|:--------------------------------------------|
| `Allow` | Matching traffic is permitted.              |
| `Block` | Matching traffic is denied.                 |

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

Indicates whether the firewall rule is active. A rule that exists but has `enabled: false` doesn't
affect network traffic.

### profiles

<details><summary>Expand for <code>profiles</code> property metadata</summary>

```yaml
Type              : array
ItemsType         : string
ItemsMustBeUnique : false
IsRequired        : false
IsKey             : false
IsReadOnly        : false
IsWriteOnly       : false
Enum              : [Domain, Private, Public, All]
```

</details>

The network location profiles for which the rule is active. Specifying `All` is equivalent to
specifying all three individual profiles. When all three individual profiles (`Domain`, `Private`,
`Public`) are set, the resource normalizes them to `All`.

### grouping

<details><summary>Expand for <code>grouping</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The grouping string that associates the rule with a named feature or application group, shown in
the Windows Firewall console as the **Program** or **Group** column.

### interfaceTypes

<details><summary>Expand for <code>interfaceTypes</code> property metadata</summary>

```yaml
Type              : array
ItemsType         : string
ItemsMustBeUnique : false
IsRequired        : false
IsKey             : false
IsReadOnly        : false
IsWriteOnly       : false
Enum              : [RemoteAccess, Wireless, Lan, All]
```

</details>

The network interface types for which the rule applies. Specifying `All` is equivalent to
specifying every interface type.

| Value          | Description                                    |
|:---------------|:-----------------------------------------------|
| `RemoteAccess` | The rule applies to remote access connections. |
| `Wireless`     | The rule applies to wireless connections.      |
| `Lan`          | The rule applies to LAN connections.           |
| `All`          | The rule applies to all interface types.       |

### edgeTraversal

<details><summary>Expand for <code>edgeTraversal</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

Indicates whether edge traversal is enabled for the rule. When `true`, traffic routed through
Network Address Translation (NAT) edge devices can pass through this rule.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": ["rules"],
  "properties": {
    "rules": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "properties": {
          "name":            { "type": "string" },
          "_exist":          { "type": "boolean" },
          "description":     { "type": "string" },
          "applicationName": { "type": "string" },
          "serviceName":     { "type": "string" },
          "protocol":        { "type": "integer" },
          "localPorts":      { "type": "string" },
          "remotePorts":     { "type": "string" },
          "localAddresses":  { "type": "string" },
          "remoteAddresses": { "type": "string" },
          "direction":       { "type": "string", "enum": ["Inbound", "Outbound"] },
          "action":          { "type": "string", "enum": ["Allow", "Block"] },
          "enabled":         { "type": "boolean" },
          "profiles": {
            "type": "array",
            "items": { "type": "string", "enum": ["Domain", "Private", "Public", "All"] }
          },
          "grouping": { "type": "string" },
          "interfaceTypes": {
            "type": "array",
            "items": { "type": "string", "enum": ["RemoteAccess", "Wireless", "Lan", "All"] }
          },
          "edgeTraversal": { "type": "boolean" }
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
- [3](#exit-code-3) - Firewall error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because required arguments were missing or the operation
name was not recognized.

### Exit code 2

Indicates the resource operation failed because the JSON input could not be deserialized into a
valid `FirewallRuleList` instance.

### Exit code 3

Indicates the resource operation failed due to an error raised by the Windows Firewall COM API,
or the result could not be serialized.

## See also

- [Microsoft.Windows/Registry resource][05]
- [Microsoft.Windows/Service resource][06]
- [DSC resource capabilities][01]
- [DSC resource properties][02]

<!-- Link definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ../../../../../concepts/resources/properties.md
[03]: ./examples/get-firewall-rules.md
[04]: ./examples/configure-firewall-rules.md
[05]: ../Registry/index.md
[06]: ../Service/index.md
