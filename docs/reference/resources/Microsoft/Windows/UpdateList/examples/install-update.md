---
description: Show how to preview and install a matching Windows Update with Microsoft.Windows/UpdateList.
ms.date:     09/20/2026
ms.topic:    reference
title:       Install a matching update
---

# Install a matching update

This example shows how to preview and install an update with the
`Microsoft.Windows/UpdateList` resource. Run the commands in an elevated PowerShell session.

## Preview the installation

Use [dsc resource set][01] with the `--what-if` option to see the projected change without
installing the update:

```powershell
dsc resource set --file install-update.config.dsc.yaml --what-if
```

The response includes a `executionType: whatIf` message for the update:

```yaml
executionInformation:                                                                                                               
  duration: PT12.0321456S
  endDatetime: 2026-09-25T13:30:16.134795800-07:00
  executionType: whatIf
  operation: set
  securityContext: elevated
  startDatetime: 2026-09-25T13:30:04.102650200-07:00
  version: 3.4.0-preview.1
metadata:
  Microsoft.DSC:
    duration: PT12.032141S
    endDatetime: 2026-09-25T13:30:16.134791200-07:00
    executionType: whatIf
    operation: set
    securityContext: elevated
    startDatetime: 2026-09-25T13:30:04.102650200-07:00
    version: 3.4.0-preview.1
results:

```

## Install the update

Remove `--what-if` to download and install the matching update:

```powershell
dsc resource set --file install-update.config.dsc.yaml
```
```yaml

executionInformation:                                
  duration: PT16.5111979S                            
  endDatetime: 2026-09-25T13:32:52.271304500-07:00   
  executionType: actual
  operation: set
  securityContext: elevated
  startDatetime: 2026-09-25T13:32:35.760106600-07:00 
  version: 3.4.0-preview.1
metadata:
  Microsoft.DSC:
    duration: PT16.5111935S
    endDatetime: 2026-09-25T13:32:52.271300100-07:00 
    executionType: actual
    operation: set
    securityContext: elevated
    startDatetime: 2026-09-25T13:32:35.760106600-07:00
    version: 3.4.0-preview.1
results:
- executionInformation:
    duration: PT14.8847236S
  metadata:
    Microsoft.DSC:
  type: Microsoft.Windows/UpdateList
  result:
    beforeState:
      updates:
      - description: Install or update to PowerShell version v7.6.6 (x64)
        id: ad9fa3fd-290d-4f3c-a820-69956938b5f2
        installationBehavior: CanRequestReboot
        isInstalled: true
        isUninstallable: false
        kbArticleIds: []
        recommendedHardDiskSpace: 0
        securityBulletinIds: []
        title: PowerShell v7.6.6 (x64)
        updateType: Software
    afterState:
      updates:
      - description: Install or update to PowerShell version v7.6.6 (x64)
        id: ad9fa3fd-290d-4f3c-a820-69956938b5f2
        installationBehavior: CanRequestReboot
        isInstalled: true
        isUninstallable: false
        kbArticleIds: []
        recommendedHardDiskSpace: 0
        securityBulletinIds: []
        title: PowerShell v7.6.6 (x64)
        updateType: Software
    changedProperties: []
messages: []
hadErrors: false

```

The operation can require a restart after installation.

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/set.md
