---
description: >
  Examples showing how to export and filter Windows Features on Demand (capabilities) using the
  Microsoft.Windows/FeatureOnDemandList resource.
ms.date:     04/21/2026
ms.topic:    reference
title:       Export Features on Demand
---

# Export Features on Demand

This example shows how you can use the `Microsoft.Windows/FeatureOnDemandList` resource to
enumerate Windows Features on Demand (capabilities) on a system, optionally filtering the results
by identity, state, display name, or description.

> [!IMPORTANT]
> All operations with `Microsoft.Windows/FeatureOnDemandList` require an elevated (administrator)
> session. Run your terminal as administrator before executing these commands.

## Export all capabilities

To retrieve a complete list of all capabilities on the system, use the [dsc resource export][01]
command without any input.

```powershell
dsc resource export --resource Microsoft.Windows/FeatureOnDemandList
```

DSC returns a configuration document that includes all capabilities and their current states:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/FeatureOnDemandList
  type: Microsoft.Windows/FeatureOnDemandList
  properties:
    capabilities:
    - identity: OpenSSH.Client~~~~0.0.1.0
      state: Installed
    - identity: OpenSSH.Server~~~~0.0.1.0
      state: NotPresent
    - identity: Language.Basic~~~en-US~0.0.1.0
      state: Installed
    # ... additional capabilities
```

> [!NOTE]
> When exporting without filters, the resource uses a fast enumeration path that returns only
> `identity` and `state` for each capability. To retrieve additional properties such as
> `displayName`, `description`, `downloadSize`, and `installSize`, use an export filter as shown
> in the examples below.

## Export only installed capabilities

To list only the capabilities currently installed on the system, provide a filter with
`state: Installed`.

```powershell
$filter = @{
    capabilities = @(
        @{ state = 'Installed' }
    )
} | ConvertTo-Json -Depth 3

dsc resource export --resource Microsoft.Windows/FeatureOnDemandList --input $filter
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/FeatureOnDemandList
  type: Microsoft.Windows/FeatureOnDemandList
  properties:
    capabilities:
    - identity: OpenSSH.Client~~~~0.0.1.0
      state: Installed
    - identity: Language.Basic~~~en-US~0.0.1.0
      state: Installed
    # ... additional installed capabilities
```

## Export capabilities by identity pattern

You can filter capabilities by identity using wildcard (`*`) patterns. The match is
case-insensitive.

```powershell
$filter = @{
    capabilities = @(
        @{ identity = 'OpenSSH*' }
    )
} | ConvertTo-Json -Depth 3

dsc resource export --resource Microsoft.Windows/FeatureOnDemandList --input $filter
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/FeatureOnDemandList
  type: Microsoft.Windows/FeatureOnDemandList
  properties:
    capabilities:
    - identity: OpenSSH.Client~~~~0.0.1.0
      state: Installed
    - identity: OpenSSH.Server~~~~0.0.1.0
      state: NotPresent
```

## Export capabilities with full details

To retrieve full details including `displayName`, `description`, `downloadSize`, and
`installSize`, include those properties as filters. A wildcard (`*`) in a filter property matches
all values for that field and triggers the full-info lookup.

```powershell
$filter = @{
    capabilities = @(
        @{
            identity    = 'OpenSSH*'
            displayName = '*'
        }
    )
} | ConvertTo-Json -Depth 3

dsc resource export --resource Microsoft.Windows/FeatureOnDemandList --input $filter
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/FeatureOnDemandList
  type: Microsoft.Windows/FeatureOnDemandList
  properties:
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

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/export.md
