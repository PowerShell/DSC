---
description: >
  Examples showing how to export and filter Windows Optional features using the
  Microsoft.Windows/OptionalFeatureList resource.
ms.date:     04/21/2026
ms.topic:    reference
title:       Export optional features
---

# Export optional features

This example shows how you can use the `Microsoft.Windows/OptionalFeatureList` resource to
enumerate Windows Optional features on a system, optionally filtering the results by name, state,
display name, or description.

> [!IMPORTANT]
> All operations with `Microsoft.Windows/OptionalFeatureList` require an elevated (administrator)
> session. Run your terminal as administrator before executing these commands.

## Export all optional features

To retrieve a complete list of all optional features on the system, use the [dsc resource export][01]
command without any input.

```powershell
dsc resource export --resource Microsoft.Windows/OptionalFeatureList
```

DSC returns a configuration document that includes all optional features and their current states:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/OptionalFeatureList
  type: Microsoft.Windows/OptionalFeatureList
  properties:
    features:
    - featureName: TFTP
      state: NotPresent
    - featureName: TelnetClient
      state: NotPresent
    - featureName: Containers-DisposableClientVM
      state: NotPresent
    # ... additional features
```

> [!NOTE]
> When exporting without filters, the resource uses a fast enumeration path that returns only
> `featureName` and `state` for each feature. To retrieve additional properties such as
> `displayName` and `description`, use an export filter as shown in the examples below.

## Export only installed features

To list only the features that are currently enabled, provide a filter with `state: Installed`.

```powershell
$filter = @{
    features = @(
        @{ state = 'Installed' }
    )
} | ConvertTo-Json -Depth 3

dsc resource export --resource Microsoft.Windows/OptionalFeatureList --input $filter
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/OptionalFeatureList
  type: Microsoft.Windows/OptionalFeatureList
  properties:
    features:
    - featureName: NetFx4-AdvSrvs
      state: Installed
    - featureName: WCF-Services45
      state: Installed
    # ... additional installed features
```

## Export features by name pattern

You can filter features by name using wildcard (`*`) patterns. The match is case-insensitive.

```powershell
$filter = @{
    features = @(
        @{ featureName = 'Hyper-V*' }
    )
} | ConvertTo-Json -Depth 3

dsc resource export --resource Microsoft.Windows/OptionalFeatureList --input $filter
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/OptionalFeatureList
  type: Microsoft.Windows/OptionalFeatureList
  properties:
    features:
    - featureName: Microsoft-Hyper-V
      state: Installed
    - featureName: Microsoft-Hyper-V-Management-Clients
      state: Installed
    - featureName: Microsoft-Hyper-V-Management-PowerShell
      state: Installed
    - featureName: Microsoft-Hyper-V-Tools-All
      state: Installed
```

## Export features with full details

To retrieve full details including `displayName` and `description`, include those properties as
filters. An empty string (`""`) or a wildcard (`*`) matches all values for that field.

```powershell
$filter = @{
    features = @(
        @{
            featureName = 'TelnetClient'
            displayName = '*'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource export --resource Microsoft.Windows/OptionalFeatureList --input $filter
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/OptionalFeatureList
  type: Microsoft.Windows/OptionalFeatureList
  properties:
    features:
    - featureName: TelnetClient
      state: NotPresent
      displayName: Telnet Client
      description: Includes Telnet Client
      restartRequired: No
```

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/export.md
