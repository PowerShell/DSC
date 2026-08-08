---
description: >
  Example showing how to use the Microsoft.Windows/FirewallRuleList resource in a DSC configuration
  document to create and enforce Windows Firewall rules.
ms.date:     05/09/2026
ms.topic:    reference
title:       Configure firewall rules
---

# Configure firewall rules

This example shows how you can use the `Microsoft.Windows/FirewallRuleList` resource in a DSC
configuration document to create and enforce multiple Windows Firewall rules in a single operation.

> [!IMPORTANT]
> **Set** operations for this resource require an elevated (administrator) process context. Run
> your terminal or PowerShell session as Administrator before using `dsc config set`.

## Definition

The configuration document for this example defines one instance of the `FirewallRuleList`
resource that manages two rules:

- **DscDemo - Custom App (TCP-In)** — allows inbound TCP traffic on port 8080 for a custom
  application, active on the Domain and Private profiles.
- **DscDemo - Block Telnet (TCP-Out)** — blocks all outbound TCP connections to port 23 (Telnet)
  on all profiles.

:::code language="yaml" source="firewall.config.dsc.yaml":::

Copy the configuration document and save it as `firewall.config.dsc.yaml`.

## Test the configuration

To see whether the rules already exist, use the [dsc config test][01] command.

```powershell
dsc config test --file ./firewall.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0888807S
  metadata:
    Microsoft.DSC:
      duration: PT0.0888807S
  name: Application firewall rules
  type: Microsoft.Windows/FirewallRuleList
  result:
    desiredState:
      rules:
      - name: DscDemo - Custom App (TCP-In)
        description: Allow inbound TCP traffic on port 8080 for the custom app.
        protocol: 6
        localPorts: '8080'
        direction: Inbound
        action: Allow
        enabled: true
        profiles:
        - Domain
        - Private
      - name: DscDemo - Block Telnet (TCP-Out)
        description: Block all outbound Telnet connections.
        protocol: 6
        remotePorts: '23'
        direction: Outbound
        action: Block
        enabled: true
        profiles:
        - All
    actualState:
      rules:
      - name: DscDemo - Custom App (TCP-In)
        _exist: false
        description: Allow inbound TCP traffic on port 8080 for the custom app.
        protocol: 6
        localPorts: '8080'
        direction: Inbound
        action: Allow
        enabled: true
        profiles:
        - Domain
        - Private
      - name: DscDemo - Block Telnet (TCP-Out)
        _exist: false
        description: Block all outbound Telnet connections.
        protocol: 6
        remotePorts: '23'
        direction: Outbound
        action: Block
        enabled: true
        profiles:
        - All
    inDesiredState: false
    differingProperties:
    - rules
messages: []
hadErrors: false
```

Neither rule exists in the firewall store, so both entries in `actualState` show `_exist: false`.
Because the actual state differs from the desired state, `inDesiredState` is `false` and `rules`
is listed in `differingProperties`.

## Set the configuration

To enforce the desired state and create both rules, use the [dsc config set][02] command.

```powershell
dsc config set --file ./firewall.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.288497S
  metadata:
    Microsoft.DSC:
      duration: PT0.288497S
  name: Application firewall rules
  type: Microsoft.Windows/FirewallRuleList
  result:
    beforeState:
      rules:
      - name: DscDemo - Custom App (TCP-In)
        _exist: false
        description: Allow inbound TCP traffic on port 8080 for the custom app.
        protocol: 6
        localPorts: '8080'
        direction: Inbound
        action: Allow
        enabled: true
        profiles:
        - Domain
        - Private
      - name: DscDemo - Block Telnet (TCP-Out)
        _exist: false
        description: Block all outbound Telnet connections.
        protocol: 6
        remotePorts: '23'
        direction: Outbound
        action: Block
        enabled: true
        profiles:
        - All
    afterState:
      rules:
      - name: DscDemo - Custom App (TCP-In)
        description: Allow inbound TCP traffic on port 8080 for the custom app.
        protocol: 6
        localPorts: '8080'
        remotePorts: '*'
        localAddresses: '*'
        remoteAddresses: '*'
        direction: Inbound
        action: Allow
        enabled: true
        profiles:
        - Domain
        - Private
        interfaceTypes:
        - All
        edgeTraversal: false
      - name: DscDemo - Block Telnet (TCP-Out)
        description: Block all outbound Telnet connections.
        protocol: 6
        localPorts: '*'
        remotePorts: '23'
        localAddresses: '*'
        remoteAddresses: '*'
        direction: Outbound
        action: Block
        enabled: true
        profiles:
        - All
        interfaceTypes:
        - All
        edgeTraversal: false
    changedProperties:
    - rules
messages: []
hadErrors: false
```

Both rules were created. The `beforeState` shows both rules with `_exist: false`, confirming they
didn't exist before the operation. The `afterState` shows the complete configuration read back
from the firewall store after creation, including `interfaceTypes: [All]` and
`edgeTraversal: false` filled in by Windows. `changedProperties` lists `rules` because the rules
array changed.

## Cleanup

To return your system to its original state:

1. Save the following configuration as `firewall.cleanup.config.dsc.yaml`.

   :::code language="yaml" source="firewall.cleanup.config.dsc.yaml":::

2. Use the **Set** operation on the cleanup configuration document.

   ```powershell
   dsc config set --file ./firewall.cleanup.config.dsc.yaml
   ```

<!-- Link definitions -->
[01]: ../../../../../cli/config/test.md
[02]: ../../../../../cli/config/set.md
