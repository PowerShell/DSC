---
description: >
  Examples showing how to enable and disable Windows Optional features using the
  Microsoft.Windows/OptionalFeatureList resource.
ms.date:     04/21/2026
ms.topic:    reference
title:       Enable and disable optional features
---

# Enable and disable optional features

This example shows how you can use the `Microsoft.Windows/OptionalFeatureList` resource to enable
and disable Windows Optional features on a system. The examples use `TelnetClient` as a
representative feature name.

> [!IMPORTANT]
> All operations with `Microsoft.Windows/OptionalFeatureList` require an elevated (administrator)
> session. Run your terminal as administrator before executing these commands.

## Enable an optional feature

To enable an optional feature, set its `state` to `Installed` and use the [dsc resource set][01]
command.

```powershell
$instance = @{
    features = @(
        @{
            featureName = 'TelnetClient'
            state       = 'Installed'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource set --resource Microsoft.Windows/OptionalFeatureList --input $instance
```

When the resource enables the feature, DSC returns the updated state:

```yaml
beforeState:
  features:
  - featureName: TelnetClient
    state: NotPresent
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
afterState:
  features:
  - featureName: TelnetClient
    state: Installed
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
changedProperties:
- features
```

If a system restart is required to complete the operation, the response includes a
`_restartRequired` property at the top level:

```yaml
afterState:
  _restartRequired:
  - system: MYCOMPUTER
  features:
  - featureName: SomeFeature
    state: InstallPending
    ...
changedProperties:
- features
```

## Disable an optional feature (keep payload staged)

To disable a feature while keeping the feature payload on disk (so it can be re-enabled quickly
without source media), set `state` to `NotPresent`.

```powershell
$instance = @{
    features = @(
        @{
            featureName = 'TelnetClient'
            state       = 'NotPresent'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource set --resource Microsoft.Windows/OptionalFeatureList --input $instance
```

```yaml
beforeState:
  features:
  - featureName: TelnetClient
    state: Installed
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
afterState:
  features:
  - featureName: TelnetClient
    state: NotPresent
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
changedProperties:
- features
```

## Disable an optional feature and remove its payload

To disable a feature and completely remove its payload from disk, set `state` to `Removed`. This
frees disk space but requires source media (or Windows Update access) to re-enable the feature
later.

```powershell
$instance = @{
    features = @(
        @{
            featureName = 'TelnetClient'
            state       = 'Removed'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource set --resource Microsoft.Windows/OptionalFeatureList --input $instance
```

```yaml
beforeState:
  features:
  - featureName: TelnetClient
    state: Installed
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
afterState:
  features:
  - featureName: TelnetClient
    state: Removed
    displayName: Telnet Client
    description: Includes Telnet Client
    restartRequired: No
changedProperties:
- features
```

## Manage multiple features in a single operation

You can enable or disable multiple features in a single **Set** call by specifying multiple entries
in the `features` array.

```powershell
$instance = @{
    features = @(
        @{
            featureName = 'TelnetClient'
            state       = 'Installed'
        }
        @{
            featureName = 'TFTP'
            state       = 'NotPresent'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource set --resource Microsoft.Windows/OptionalFeatureList --input $instance
```

## Use in a configuration document

You can also use the resource in a DSC configuration document to declaratively manage optional
features across a system.

```yaml
# optional-features.config.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Enable Telnet Client
    type: Microsoft.Windows/OptionalFeatureList
    properties:
      features:
        - featureName: TelnetClient
          state: Installed
        - featureName: TFTP
          state: NotPresent
```

Apply the configuration with the [dsc config set][02] command:

```powershell
dsc config set --file ./optional-features.config.dsc.yaml
```

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/set.md
[02]: ../../../../../../cli/config/set.md
