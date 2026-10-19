---
description: >
  Example showing how to use the Microsoft.Windows/AuditPolicyList resource with the dsc resource
  commands to retrieve, test and export the auditing of several subcategories.
ms.date:     10/19/2026
ms.topic:    reference
title:       Get audit policy list
---

# Get audit policy list

This example shows how you can use the `Microsoft.Windows/AuditPolicyList` resource to retrieve
and test the auditing of several subcategories at once, and to export the full audit policy as a
single instance.

> [!IMPORTANT]
> All operations of this resource require an elevated (administrator) process context. Run your
> terminal or PowerShell session as Administrator before using the `dsc resource` commands.

## Get the auditing of several subcategories

To retrieve the current auditing of several subcategories, use the [dsc resource get][01] command.
List one entry per subcategory; the only required property of an entry is `subcategory`, which
can be the English name of the subcategory or its GUID.

```powershell
$instance = @{
    policies = @(
        @{ subcategory = 'Logon' }
        @{ subcategory = '{0CCE923F-69AE-11D9-BED3-505054503030}' }
    )
} | ConvertTo-Json -Depth 3

dsc resource get --resource Microsoft.Windows/AuditPolicyList --input $instance
```

```yaml
actualState:
  policies:
  - subcategory: Logon
    success: true
    failure: true
    guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
    category: Logon/Logoff
  - subcategory: '{0CCE923F-69AE-11D9-BED3-505054503030}'
    success: true
    failure: false
    guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
    category: Account Logon
```

The resource returns the entries in the order you listed them, and returns each `subcategory`
exactly as you specified it, together with the read-only `guid` and `category` properties.

## Test the auditing of several subcategories

To check whether the subcategories have the auditing you expect, use the [dsc resource test][02]
command. Only the flags that each entry specifies are compared, and the instance is in the desired
state only when every entry matches.

```powershell
$instance = @{
    policies = @(
        @{ subcategory = 'Logon'; success = $true; failure = $true }
        @{ subcategory = 'Credential Validation'; failure = $true }
    )
} | ConvertTo-Json -Depth 3

dsc resource test --resource Microsoft.Windows/AuditPolicyList --input $instance
```

```yaml
desiredState:
  policies:
  - subcategory: Logon
    success: true
    failure: true
  - subcategory: Credential Validation
    failure: true
actualState:
  policies:
  - subcategory: Logon
    success: true
    failure: true
    guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
    category: Logon/Logoff
  - subcategory: Credential Validation
    success: true
    failure: false
    guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
    category: Account Logon
inDesiredState: false
differingProperties:
- policies
```

The `Logon` entry matches, but `Credential Validation` doesn't audit failures, so the instance
isn't in the desired state.

## Export the full audit policy

To export the auditing of every subcategory, use the [dsc resource export][03] command. The
output is a configuration document with a single instance that lists every subcategory, grouped
by category, which you can use as the starting point for a baseline.

```powershell
dsc resource export --resource Microsoft.Windows/AuditPolicyList
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/AuditPolicyList-0
  type: Microsoft.Windows/AuditPolicyList
  properties:
    policies:
    - subcategory: Security State Change
      success: true
      failure: false
      guid: '{0CCE9210-69AE-11D9-BED3-505054503030}'
      category: System
    - subcategory: Security System Extension
      success: false
      failure: false
      guid: '{0CCE9211-69AE-11D9-BED3-505054503030}'
      category: System
    # Elided for brevity
```

<!-- Link definitions -->
[01]: ../../../../../../cli/resource/get.md
[02]: ../../../../../../cli/resource/test.md
[03]: ../../../../../../cli/resource/export.md
