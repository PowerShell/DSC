---
description: Discover installed CSP bridge resources and inspect their current state.
ms.date: 10/25/2026
ms.topic: reference
title: Discover and inspect CSP resources
---

# Discover and inspect CSP resources

## Discover bridge classes

List resources supported by the CSP adapter:

```powershell
dsc resource list --adapter Microsoft.Windows.Adapter/ConfigurationServiceProvider
```

Each JSON Line identifies an installed class. A representative entry includes:

```yaml
type: Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02
kind: resource
version: 0.1.0
capabilities: [get, test, export, set, setHandlesExist]
requireAdapter: Microsoft.Windows.Adapter/ConfigurationServiceProvider
```

Class availability, properties, and access requirements depend on the target Windows version.
Discovery lists both device and user classes; it does not grant access to either partition.

## Read a device policy

Run the following command as LocalSystem, not just an elevated administrator:

```powershell
$json = @{
    ParentID = './Vendor/MSFT/Policy/Config'
    InstanceID = 'WiFi'
} | ConvertTo-Json -Compress
$json | dsc resource get -r Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02 -f -
```

For a configured instance, the result includes the keys, readable properties, and `_exist: true`.
If the instance is absent, the result is:

```yaml
actualState:
  InstanceID: WiFi
  ParentID: ./Vendor/MSFT/Policy/Config
  _exist: false
```

An access failure produces an error, not this absent result.
