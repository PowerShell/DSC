---
description: Configure user registry preferences and administrator-controlled registry policy.
ms.date:     11/22/2026
ms.topic:    reference
title:       Configure registry preferences and policy
---

# Configure registry preferences and policy

## Configure current-user preferences

Run the following commands in PowerShell on any supported platform:

```powershell
$settings = @{
    registries = @(
        @{ public = @{ location = 'registry.example.com' } }
        @{ private = @{
            location = 'private.example.com'
            authentication = @{ secretName = 'registry-token'; secretVault = 'registry-vault' }
        } }
    )
}
$settings | ConvertTo-Json -Depth 10 |
    dsc resource set -r Microsoft.Registry.OpenContainerInitiative/RegistryList -f -

dsc resource list
```

The discovery extension requests `registry-token` from a loaded Secret provider only when
discovering `private`. Resource listing returns the registry's friendly name in `registry`.
Resource manifests are downloaded; implementation ZIPs are not.

## Enforce a machine policy registry

Run the following in an administrator session:

```powershell
@{
    scope = 'allUsers'
    policy = $true
    registries = @(@{ corporate = @{ location = 'approved.example.com' } })
} | ConvertTo-Json -Depth 10 |
    dsc resource set -r Microsoft.Registry.OpenContainerInitiative/RegistryList -f -
```

An entry named `corporate` in either preference file cannot override this policy. Distinct user
registries remain discoverable. To manage machine preferences instead, use the same scope with
`policy = $false`. Keep both machine files protected by administrator-controlled permissions.

## Clear current-user preferences

```powershell
'{"registries":[]}' |
    dsc resource set -r Microsoft.Registry.OpenContainerInitiative/RegistryList -f -
```

Machine preferences and machine policy remain unchanged.
