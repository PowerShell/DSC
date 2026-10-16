---
description: >
  Example showing how to use the Microsoft.Windows/AuditPolicy resource with the dsc resource
  commands to retrieve and export the audit policy.
ms.date:     10/16/2026
ms.topic:    reference
title:       Get audit policy
---

# Get audit policy

This example shows how you can use the `Microsoft.Windows/AuditPolicy` resource to retrieve the
auditing of one subcategory and to export the auditing of every subcategory.

> [!IMPORTANT]
> All operations of this resource require an elevated (administrator) process context. Run your
> terminal or PowerShell session as Administrator before using the `dsc resource` commands.

## Get the auditing of a subcategory

To retrieve the current auditing of a subcategory, use the [dsc resource get][01] command. The
only required property is `subcategory`. You can specify the English name of the subcategory or
its GUID.

```powershell
dsc resource get --resource Microsoft.Windows/AuditPolicy --input '{"subcategory":"Logon"}'
```

```yaml
actualState:
  subcategory: Logon
  success: true
  failure: true
  guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
  category: Logon/Logoff
```

The resource returns the `subcategory` exactly as you specified it, together with the read-only
`guid` and `category` properties. The following command retrieves the same subcategory by GUID:

```powershell
dsc resource get --resource Microsoft.Windows/AuditPolicy --input '{"subcategory":"{0CCE9215-69AE-11D9-BED3-505054503030}"}'
```

```yaml
actualState:
  subcategory: '{0CCE9215-69AE-11D9-BED3-505054503030}'
  success: true
  failure: true
  guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
  category: Logon/Logoff
```

## Test the auditing of a subcategory

To check whether a subcategory has the auditing you expect, use the [dsc resource test][02]
command. Only the flags you specify are compared.

```powershell
dsc resource test --resource Microsoft.Windows/AuditPolicy --input '{"subcategory":"Credential Validation","failure":true}'
```

```yaml
desiredState:
  subcategory: Credential Validation
  failure: true
actualState:
  subcategory: Credential Validation
  success: true
  failure: false
  guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
  category: Account Logon
inDesiredState: false
differingProperties:
- failure
```

## Export the full audit policy

To export the auditing of every subcategory, use the [dsc resource export][03] command. The
output is a configuration document with one instance per subcategory, grouped by category, that
you can use as the starting point for a baseline.

```powershell
dsc resource export --resource Microsoft.Windows/AuditPolicy
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/AuditPolicy-0
  type: Microsoft.Windows/AuditPolicy
  properties:
    subcategory: Security State Change
    success: true
    failure: false
    guid: '{0CCE9210-69AE-11D9-BED3-505054503030}'
    category: System
- name: Microsoft.Windows/AuditPolicy-1
  type: Microsoft.Windows/AuditPolicy
  properties:
    subcategory: Security System Extension
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
