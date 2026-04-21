---
description: >
  Examples showing how to retrieve the current state of Windows Optional Features using the
  Microsoft.Windows/OptionalFeatureList resource.
ms.date:     04/21/2026
ms.topic:    reference
title:       Get optional feature state
---

# Get optional feature state

This example shows how you can use the `Microsoft.Windows/OptionalFeatureList` resource to retrieve
the current state of Windows Optional Features. The examples use `TelnetClient` as a
representative feature name.

> [!IMPORTANT]
> All operations with `Microsoft.Windows/OptionalFeatureList` require an elevated (administrator)
> session. Run your terminal as administrator before executing these commands.

## Get a single feature

The following snippet shows how to retrieve the state of the `TelnetClient` feature using the
[dsc resource get][01] command.

```powershell
$instance = @{
    features = @(
        @{ featureName = 'TelnetClient' }
    )
} | ConvertTo-Json -Depth 3

dsc resource get --resource Microsoft.Windows/OptionalFeatureList --input $instance
```

When the feature is disabled, DSC returns output similar to the following:

```yaml
actualState:
  features:
  - featureName: TelnetClient
    state: Disabled
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
```

When the feature is enabled, the `state` field reads `Installed`:

```yaml
actualState:
  features:
  - featureName: TelnetClient
    state: Installed
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
```

## Get multiple features in a single request

You can retrieve the state of multiple features in a single call by including multiple entries in
the `features` array.

```powershell
$instance = @{
    features = @(
        @{ featureName = 'TelnetClient' }
        @{ featureName = 'TFTP' }
    )
} | ConvertTo-Json -Depth 3

dsc resource get --resource Microsoft.Windows/OptionalFeatureList --input $instance
```

DSC returns the state of all requested features in a single response:

```yaml
actualState:
  features:
  - featureName: TelnetClient
    state: Disabled
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
  - featureName: TFTP
    state: Disabled
    displayName: TFTP Client
    description: Includes TFTP Client
    restartRequired: No
```

## Get a non-existent feature

When you request a feature name that is not recognized by DISM, the resource returns `_exist: false`
instead of raising an error.

```powershell
$instance = @{
    features = @(
        @{ featureName = 'NonExistent-Feature-XYZ' }
    )
} | ConvertTo-Json -Depth 3

dsc resource get --resource Microsoft.Windows/OptionalFeatureList --input $instance
```

```yaml
actualState:
  features:
  - featureName: NonExistent-Feature-XYZ
    _exist: false
```

The `_exist: false` response indicates the feature name is not recognized by DISM on this system.

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/get.md
