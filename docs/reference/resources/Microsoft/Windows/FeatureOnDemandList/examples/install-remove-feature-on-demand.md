---
description: >
  Examples showing how to install and remove Windows features on demand (capabilities) using the
  Microsoft.Windows/FeatureOnDemandList resource.
ms.date:     04/21/2026
ms.topic:    reference
title:       Install and remove features on demand
---

# Install and remove features on demand

This example shows how you can use the `Microsoft.Windows/FeatureOnDemandList` resource to install
and remove Windows features on demand (capabilities). The examples use
`OpenSSH.Client~~~~0.0.1.0` as a representative capability identity.

> [!IMPORTANT]
> All operations with `Microsoft.Windows/FeatureOnDemandList` require an elevated (administrator)
> session. Run your terminal as administrator before executing these commands.

> [!NOTE]
> Installing a capability may require internet access or an appropriately configured Windows Update
> or WSUS source. Installing large capabilities may take several minutes to complete.

## Install a capability

To install a capability, set its `state` to `Installed` and use the [dsc resource set][01]
command.

```powershell
$instance = @{
    capabilities = @(
        @{
            identity = 'OpenSSH.Client~~~~0.0.1.0'
            state    = 'Installed'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource set --resource Microsoft.Windows/FeatureOnDemandList --input $instance
```

When the resource installs the capability, DSC returns the updated state:

```yaml
beforeState:
  capabilities:
  - identity: OpenSSH.Client~~~~0.0.1.0
    state: NotPresent
    displayName: OpenSSH Client
    description: Open SSH-based secure shell (SSH) client...
    downloadSize: 4026000
    installSize: 4894720
afterState:
  capabilities:
  - identity: OpenSSH.Client~~~~0.0.1.0
    state: Installed
    displayName: OpenSSH Client
    description: Open SSH-based secure shell (SSH) client...
    downloadSize: 0
    installSize: 4894720
changedProperties:
- capabilities
```

If a system restart is required to complete the installation, the response includes a
`_restartRequired` property at the top level:

```yaml
afterState:
  _restartRequired:
  - system: MYCOMPUTER
  capabilities:
  - identity: SomeCapability~~~~0.0.1.0
    state: InstallPending
    ...
changedProperties:
- capabilities
```

## Remove a capability

To remove a capability, set its `state` to `NotPresent` and use the [dsc resource set][01] command.

```powershell
$instance = @{
    capabilities = @(
        @{
            identity = 'OpenSSH.Client~~~~0.0.1.0'
            state    = 'NotPresent'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource set --resource Microsoft.Windows/FeatureOnDemandList --input $instance
```

```yaml
beforeState:
  capabilities:
  - identity: OpenSSH.Client~~~~0.0.1.0
    state: Installed
    displayName: OpenSSH Client
    description: Open SSH-based secure shell (SSH) client...
    downloadSize: 0
    installSize: 4894720
afterState:
  capabilities:
  - identity: OpenSSH.Client~~~~0.0.1.0
    state: NotPresent
    displayName: OpenSSH Client
    description: Open SSH-based secure shell (SSH) client...
    downloadSize: 4026000
    installSize: 4894720
changedProperties:
- capabilities
```

## Manage multiple capabilities in a single operation

You can install or remove multiple capabilities in a single **Set** call by specifying multiple
entries in the `capabilities` array. The resource processes each entry independently.

```powershell
$instance = @{
    capabilities = @(
        @{
            identity = 'OpenSSH.Client~~~~0.0.1.0'
            state    = 'Installed'
        }
        @{
            identity = 'OpenSSH.Server~~~~0.0.1.0'
            state    = 'NotPresent'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource set --resource Microsoft.Windows/FeatureOnDemandList --input $instance
```

## Use in a configuration document

You can also use the resource in a DSC configuration document to declaratively manage capabilities
across a system.

```yaml
# features-on-demand.config.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Manage OpenSSH capabilities
    type: Microsoft.Windows/FeatureOnDemandList
    properties:
      capabilities:
        - identity: OpenSSH.Client~~~~0.0.1.0
          state: Installed
        - identity: OpenSSH.Server~~~~0.0.1.0
          state: NotPresent
```

Apply the configuration with the [dsc config set][02] command:

```powershell
dsc config set --file ./features-on-demand.config.dsc.yaml
```

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/set.md
[02]: ../../../../../../cli/config/set.md
