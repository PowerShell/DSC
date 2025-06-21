---
description: >
    Example showing how to use the Microsoft.Windows/WMI resource adapter to query
    disk information with filtering using the Win32_LogicalDisk class.

ms.date: 03/25/2025
ms.topic: reference
title: Query filtered disk information using WMI adapter
---

This example demonstrates how to use the `Microsoft.Windows/WMI` resource adapter to query disk
information from a computer using the Win32_LogicalDisk WMI class with filtering to get only
specific drives using a configuration document.

## Definition

The configuration document for this example defines one instances of the `Win32_LogicalDisk` resource.

The instance defines the properties to return in the output.

:::code language="yaml" source="logicaldisk.config.dsc.yaml":::

Copy the configuration document and save it as `logicaldisk.config.dsc.yaml`.

Before using the [dsc config get][01] command, the following section illustrates how you
can retrieve the available `Win32_LogicalDisk` properties.

## List available disk properties

To list out only the `Win32_LogicalDisk` WMI class, you can run the following command:

```powershell
dsc resource list --adapter Microsoft.Windows/WMI root.cimv2/Win32_LogicalDisk |
ConvertFrom-Json |
Select-Object -ExpandProperty properties
```

DSC returns the following information:

```text
Caption
Description
InstallDate
Name
Status
Availability
ConfigManagerErrorCode
ConfigManagerUserConfig
CreationClassName
DeviceID
ErrorCleared
ErrorDescription
LastErrorCode
PNPDeviceID
PowerManagementCapabilities
PowerManagementSupported
StatusInfo
SystemCreationClassName
SystemName
Access
BlockSize
ErrorMethodology
NumberOfBlocks
Purpose
FreeSpace
Size
Compressed
DriveType
FileSystem
MaximumComponentLength
MediaType
ProviderName
QuotasDisabled
QuotasIncomplete
QuotasRebuilding
SupportsDiskQuotas
SupportsFileBasedCompression
VolumeDirty
VolumeName
VolumeSerialNumber
```

## Query disk information with filtering

To retrieve disk information with filtering, you can create a configuration file in YAML format and use it
with the `dsc config get` command.

```powershell
dsc config get --file ./logicaldisk.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.1.0
    operation: get
    executionType: actual
    startDatetime: 2025-06-21T15:17:44.158969400+02:00
    endDatetime: 2025-06-21T15:17:54.213683700+02:00
    duration: PT10.0547143S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT5.9959229S
  name: List logical disk information
  type: root.cimv2/Win32_LogicalDisk
  result:
    actualState:
      Description: Local Fixed Disk
      Name: 'C:'
      Status: null
messages: []
hadErrors: false
```

This configuration will return only the specified properties for each fixed disk.

<!-- Link reference definitions -->
[01]: ../../../../../cli/config/get.md