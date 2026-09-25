---
description: Show how to retrieve information about a Windows Update with Microsoft.Windows/UpdateList.
ms.date:     09/20/2026
ms.topic:    reference
title:       Get information about an update
---

# Get information about an update

This example shows how to retrieve an update by its exact title with the
`Microsoft.Windows/UpdateList` resource.

## Query an update

The `title` criterion is case-insensitive and must identify one update. Use the [dsc resource
get][01] command with the configuration in [get-update.config.dsc.yaml][02].

```powershell
dsc resource get --file get-update.config.dsc.yaml
```

The result includes details such as the update ID, installation status, KB article IDs, severity,
and update type:

```yaml
    actualState:
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
```

The returned values vary depending on the updates available on the system.

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/get.md
[02]: ./get-update.config.dsc.yaml
