---
description: >
  Examples showing how to use the Microsoft.Windows/FirewallRuleList resource with DSC to retrieve
  and modify the state of Windows Firewall rules.
ms.date:     05/09/2026
ms.topic:    reference
title:       Get firewall rule state
---

# Get firewall rule state

This example shows how you can use the `Microsoft.Windows/FirewallRuleList` resource to retrieve
the current state of a Windows Firewall rule and toggle it with `dsc resource` commands.

The example uses the built-in "Remote Desktop - User Mode (TCP-In)" rule, which is present on
every Windows installation and controls whether Remote Desktop connections are allowed.

## Get the state of a firewall rule

The following snippet retrieves the current state of the Remote Desktop inbound rule.

```powershell
$instance = @{
  rules = @(@{ name = 'Remote Desktop - User Mode (TCP-In)' })
} | ConvertTo-Json -Compress

dsc resource get --resource Microsoft.Windows/FirewallRuleList --input $instance
```

When the rule exists, DSC returns its full configuration. Notice that `_exist` is absent from the
actual state — for this resource, an absent `_exist` means the rule is present.

```yaml
actualState:
  rules:
  - name: Remote Desktop - User Mode (TCP-In)
    description: Inbound rule for the Remote Desktop service to allow RDP traffic. [TCP 3389]
    protocol: 6
    localPorts: "3389"
    direction: Inbound
    action: Allow
    enabled: false
    profiles:
    - Domain
    - Private
    grouping: Remote Desktop
    interfaceTypes:
    - All
    edgeTraversal: false
```

The rule exists but `enabled: false` means it is currently inactive and not filtering traffic.

## Get the state of a rule that doesn't exist

When the named rule is not registered in the Windows Firewall store, the resource returns
`_exist: false` and omits any properties that were not provided in the
input.

```powershell
$instance = @{
  rules = @(@{ name = 'DscDemo - Custom App (TCP-In)' })
} | ConvertTo-Json -Compress

dsc resource get --resource Microsoft.Windows/FirewallRuleList --input $instance
```

```yaml
actualState:
  rules:
  - name: DscDemo - Custom App (TCP-In)
    _exist: false
```

## Enable a firewall rule

To enable the Remote Desktop rule, use the [dsc resource set][01] command with `enabled: true`.
This operation requires an elevated (administrator) terminal.

```powershell
$desired = @{
  rules = @(@{
    name    = 'Remote Desktop - User Mode (TCP-In)'
    enabled = $true
  })
} | ConvertTo-Json -Compress

dsc resource set --resource Microsoft.Windows/FirewallRuleList --input $desired
```

DSC first tests the current state and then calls the resource's `set` operation because the rule
is not in the desired state. The output shows the state before and after the change.

```yaml
beforeState:
  rules:
  - name: Remote Desktop - User Mode (TCP-In)
    description: Inbound rule for the Remote Desktop service to allow RDP traffic. [TCP 3389]
    protocol: 6
    localPorts: "3389"
    direction: Inbound
    action: Allow
    enabled: false
    profiles:
    - Domain
    - Private
    grouping: Remote Desktop
    interfaceTypes:
    - All
    edgeTraversal: false
afterState:
  rules:
  - name: Remote Desktop - User Mode (TCP-In)
    description: Inbound rule for the Remote Desktop service to allow RDP traffic. [TCP 3389]
    protocol: 6
    localPorts: "3389"
    direction: Inbound
    action: Allow
    enabled: true
    profiles:
    - Domain
    - Private
    grouping: Remote Desktop
    interfaceTypes:
    - All
    edgeTraversal: false
changedProperties:
- rules
```

The `changedProperties` field lists `rules` because the `enabled` property of the rule changed.

## Query multiple rules at once

A single **Get** call can retrieve the state of multiple rules by listing them all in the `rules`
array.

```powershell
$instance = @{
  rules = @(
    @{ name = 'Remote Desktop - User Mode (TCP-In)' }
    @{ name = 'Remote Desktop - User Mode (UDP-In)' }
  )
} | ConvertTo-Json -Compress

dsc resource get --resource Microsoft.Windows/FirewallRuleList --input $instance
```

```yaml
actualState:
  rules:
  - name: Remote Desktop - User Mode (TCP-In)
    description: Inbound rule for the Remote Desktop service to allow RDP traffic. [TCP 3389]
    protocol: 6
    localPorts: "3389"
    direction: Inbound
    action: Allow
    enabled: true
    profiles:
    - Domain
    - Private
    grouping: Remote Desktop
    interfaceTypes:
    - All
    edgeTraversal: false
  - name: Remote Desktop - User Mode (UDP-In)
    description: Inbound rule for the Remote Desktop service to allow RDP traffic. [UDP 3389]
    protocol: 17
    localPorts: "3389"
    direction: Inbound
    action: Allow
    enabled: true
    profiles:
    - Domain
    - Private
    grouping: Remote Desktop
    interfaceTypes:
    - All
    edgeTraversal: false
```

## Export all inbound allow rules

The [dsc resource export][02] command returns all registered firewall rules. You can pass an
optional filter to narrow the results. The following snippet exports only inbound rules with
the `Allow` action.

```powershell
$filter = @{
  rules = @(@{
    direction = 'Inbound'
    action    = 'Allow'
  })
} | ConvertTo-Json -Compress

dsc resource export --resource Microsoft.Windows/FirewallRuleList --input $filter
```

DSC emits one JSON object for each matching rule. Properties within a single filter entry are
ANDed together; multiple filter entries are ORed. This call requires an elevated terminal.

<!-- Link definitions -->
[01]: ../../../../../cli/resource/set.md
[02]: ../../../../../cli/resource/export.md
