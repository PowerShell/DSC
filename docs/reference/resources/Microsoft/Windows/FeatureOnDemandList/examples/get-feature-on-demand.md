---
description: >
  Examples showing how to retrieve the current state of Windows features on demand (capabilities)
  using the Microsoft.Windows/FeatureOnDemandList resource.
ms.date:     04/21/2026
ms.topic:    reference
title:       Get feature on demand state
---

# Get feature on demand state

This example shows how you can use the `Microsoft.Windows/FeatureOnDemandList` resource to
retrieve the current state of Windows features on demand (capabilities). The examples use
`OpenSSH.Client~~~~0.0.1.0` as a representative capability identity.

> [!IMPORTANT]
> All operations with `Microsoft.Windows/FeatureOnDemandList` require an elevated (administrator)
> session. Run your terminal as administrator before executing these commands.

## Find capability identity strings

Before you can get the state of a capability, you need its identity string. Use the following
command to list all capabilities and their identities:

```powershell
dism /Online /Get-Capabilities /Format:Table
```

Capability identities follow the format `CapabilityName~~~~LanguageTag~Version`, for example:

- `OpenSSH.Client~~~~0.0.1.0`
- `OpenSSH.Server~~~~0.0.1.0`
- `Language.Basic~~~en-US~0.0.1.0`

## Get a single capability

The following snippet shows how to retrieve the state of the OpenSSH client capability using the
[dsc resource get][01] command.

```powershell
$instance = @{
    capabilities = @(
        @{ identity = 'OpenSSH.Client~~~~0.0.1.0' }
    )
} | ConvertTo-Json -Depth 3

dsc resource get --resource Microsoft.Windows/FeatureOnDemandList --input $instance
```

When the capability is installed, DSC returns output similar to the following:

```yaml
actualState:
  capabilities:
  - identity: OpenSSH.Client~~~~0.0.1.0
    state: Installed
    displayName: OpenSSH Client
    description: >-
      Open SSH-based secure shell (SSH) client, required for secure key management and access
      to remote machines.
    downloadSize: 0
    installSize: 4894720
```

When the capability is not installed, the `state` field reads `NotPresent`:

```yaml
actualState:
  capabilities:
  - identity: OpenSSH.Client~~~~0.0.1.0
    state: NotPresent
    displayName: OpenSSH Client
    description: >-
      Open SSH-based secure shell (SSH) client, required for secure key management and access
      to remote machines.
    downloadSize: 4026000
    installSize: 4894720
```

## Get multiple capabilities in a single request

You can retrieve the state of multiple capabilities in a single call by including multiple entries
in the `capabilities` array.

```powershell
$instance = @{
    capabilities = @(
        @{ identity = 'OpenSSH.Client~~~~0.0.1.0' }
        @{ identity = 'OpenSSH.Server~~~~0.0.1.0' }
    )
} | ConvertTo-Json -Depth 3

dsc resource get --resource Microsoft.Windows/FeatureOnDemandList --input $instance
```

```yaml
actualState:
  capabilities:
  - identity: OpenSSH.Client~~~~0.0.1.0
    state: Installed
    displayName: OpenSSH Client
    description: Open SSH-based secure shell (SSH) client...
    downloadSize: 0
    installSize: 4894720
  - identity: OpenSSH.Server~~~~0.0.1.0
    state: NotPresent
    displayName: OpenSSH Server
    description: Open SSH-based secure shell (SSH) server...
    downloadSize: 1468500
    installSize: 1839104
```

## Get a non-existent capability

When you request a capability identity that is not recognized by DISM, the resource returns
`_exist: false` instead of raising an error.

```powershell
$instance = @{
    capabilities = @(
        @{ identity = 'NonExistent.Capability~~~~0.0.1.0' }
    )
} | ConvertTo-Json -Depth 3

dsc resource get --resource Microsoft.Windows/FeatureOnDemandList --input $instance
```

```yaml
actualState:
  capabilities:
  - identity: NonExistent.Capability~~~~0.0.1.0
    _exist: false
```

The `_exist: false` response indicates the capability identity is not recognized by DISM on this
system.

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/get.md
