---
description: Show how to export and filter Windows Updates with Microsoft.Windows/UpdateList.
ms.date:     09/20/2026
ms.topic:    reference
title:       Export and filter updates
---

# Export and filter updates

This example shows how to enumerate Windows Updates and filter the results.

## Export all updates

Use [dsc resource export][01] without an input document to retrieve all updates known to the
Windows Update Agent:

```powershell
dsc resource export --resource Microsoft.Windows/UpdateList
```
Exported: 

```yaml
    
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json                                       
contentVersion: 1.0.0
resources:
- type: Microsoft.Windows/UpdateList
  name: UpdateList-0
  executionInformation:
    duration: PT27.1805421S
  properties:
    updates:
    - description: A vulnerability exists in Microsoft XML Core Services that could allow for information disclosure because the XMLHTTP ActiveX control incorrectly interprets an HTTP server-side redirect.
      id: 07609d43-d518-4e77-856e-d1b316d1b8a8
      installationBehavior: CanRequestReboot
      isInstalled: true
      isUninstallable: false
      kbArticleIds:
      - '925673'
      msrcSeverity: Critical
      recommendedHardDiskSpace: 0
      securityBulletinIds:
      - MS06-061
      title: MSXML 6.0 RTM Security Update  (925673)
      updateType: Software
    - description: This package will update Windows Defender Antivirus antimalware platform’s components on the user machine.
      id: c01629fc-64ea-45f3-b7cb-cabc7d566933
      installationBehavior: CanRequestReboot
      isInstalled: true
      isUninstallable: false
      kbArticleIds:
      - '4052623'
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: Update for Windows Defender Antivirus antimalware platform - KB4052623 (Version 4.18.2001.10)
      updateType: Software
    - description: Install or update to PowerShell version v7.3.1 (x64)
      id: 5d507f01-0c39-4d10-9d23-1ce41c30a2c6       
      installationBehavior: AlwaysRequiresReboot     
      isInstalled: true
      isUninstallable: false
      kbArticleIds:
      - '5021051'
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: PowerShell v7.3.1 (x64)
      updateType: Software
    - description: Install or update to PowerShell version v7.5.8 (x64)
      id: 8b3006c9-0943-42e3-ad0f-0dbf90c1ca4b       
      installationBehavior: CanRequestReboot
      isInstalled: true
      isUninstallable: false
      kbArticleIds: []
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: PowerShell v7.5.8 (x64)
      updateType: Software
    - description: This package will update Windows Security platform components on the user machine.     
      id: a32ca1d0-ddd4-486b-b708-d941db4f1141       
      installationBehavior: NeverReboots
      isInstalled: false
      isUninstallable: false
      kbArticleIds:
      - '5007651'
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: Update for Windows Security platform - KB5007651 (Version 10.0.29628.1000)
      updateType: Software
    - description: After the download, this tool runs one time to check your computer for infection by specific, prevalent malicious software (including Blaster, Sasser, and Mydoom) and helps remove any infection that is found. If an infection is found, the tool will display a status report the next time that you start your computer. A new version of the tool will be offered every month. If you want to manually run the tool on your computer, you can download a copy from the Microsoft Download Center, or you can run an online version from microsoft.com. This tool is not a replacement for an antivirus product. To help protect your computer, you should use an antivirus product.      
      id: cb6bf2cf-30a1-4933-9f49-cfd06d02f2d1
      installationBehavior: CanRequestReboot
      isInstalled: false
      isUninstallable: false
      kbArticleIds:
      - '890830'
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: Windows Malicious Software Removal Tool x64 - v5.145 (KB890830)
      updateType: Software
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
    - description: This package will update Microsoft Defender Antivirus antimalware platform’s components on the user machine.
      id: 60dbb22a-8747-4fc3-9df0-515642710177       
      installationBehavior: NeverReboots
      isInstalled: true
      isUninstallable: false
      kbArticleIds:
      - '4052623'
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: Update for Microsoft Defender Antivirus antimalware platform - KB4052623 (Version 4.18.26080.4) - Current Channel (Staged)
      updateType: Software
    - description: Install this update to revise the files that are used to detect viruses, spyware, and other potentially unwanted software. Once you have installed this item, it cannot be removed.
      id: f3b735bd-2e61-4fae-bd36-b9ebc8bf3fe0       
      installationBehavior: NeverReboots
      isInstalled: false
      isUninstallable: false
      kbArticleIds:
      - '2267602'
t your computer.
      id: c802a52d-8518-45c9-a0e7-1bc968ad4643
      installationBehavior: CanRequestReboot
      isInstalled: false
      isUninstallable: false
      kbArticleIds:
      - '5122871'
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: 2026-09 Security Update (KB5122871) (26100.33438)
      updateType: Software
    - description: A security issue has been identified in a Microsoft software product that could affect your system. You can help protect your system by installing this update from Microsoft. For a complete listing of the issues that are included in this update, see the associated Microsoft Knowledge Base article. After you install this update, you may have to restart your system.
      id: 8c8e74a5-b7d9-4d95-8ba2-bef03e18c9e6
      installationBehavior: CanRequestReboot
      isInstalled: false
      isUninstallable: false
      kbArticleIds:
      - '5126052'
      recommendedHardDiskSpace: 0
      securityBulletinIds: []
      title: 2026-09 .NET Framework Security Update (KB5126052)
      updateType: Software

```

## Export installed security updates

Use [export-updates.config.dsc.yaml][02] to filter for installed updates with a Critical or
Important MSRC severity:

```powershell
dsc resource export --file export-updates.config.dsc.yaml
```

The resource combines criteria in one filter with AND. Separate entries in the `updates` array are
combined with OR, and duplicate updates are returned only once.

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json

resources:
- name: Filter Windows Updates
  type: Microsoft.Windows/UpdateList
  properties:
    updates:
    - title: '*Security*'
      isInstalled: false
      updateType: 'Software'
```

Title and description filters support case-insensitive `*` wildcards. Other useful filters include
`id`, `isUninstallable`, `kbArticleIds`, `recommendedHardDiskSpace`, `securityBulletinIds`, and
`updateType`.

<!-- Link reference definitions -->
[01]: ../../../../../../cli/resource/export.md
[02]: ./export-updates.config.dsc.yaml
