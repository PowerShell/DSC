---
description: >
  Example showing how to use the Microsoft.Windows/UserRightsAssignment resource with the dsc
  resource commands to retrieve, test and export user rights assignments.
ms.date:     10/16/2026
ms.topic:    reference
title:       Get user rights assignments
---

# Get user rights assignments

This example shows how you can use the `Microsoft.Windows/UserRightsAssignment` resource to
retrieve the accounts that hold a user right, test whether a right is assigned as expected, and
export every right on the system.

> [!IMPORTANT]
> All operations of this resource require an elevated (administrator) process context. Run your
> terminal or PowerShell session as Administrator before using the `dsc resource` commands.

## Get the holders of a right

To retrieve the accounts that hold a right, use the [dsc resource get][01] command. The only
required property is `right`.

```powershell
dsc resource get --resource Microsoft.Windows/UserRightsAssignment --input '{"right":"SeServiceLogonRight"}'
```

```yaml
actualState:
  right: SeServiceLogonRight
  identity:
  - NT SERVICE\ALL SERVICES
  - NT VIRTUAL MACHINE\Virtual Machines
```

The resource returns every holder as `DOMAIN\Name`. Accounts without a domain, such as `Everyone`,
are returned by name only, and a SID that no longer resolves to an account is returned as the SID
string.

## Test the holders of a right

To check whether a right is assigned the way you expect, use the [dsc resource test][02] command.
The resource compares accounts by SID, so you can write the identities in any accepted form. The
following command checks that `Guests` is the only account denied batch logon:

```powershell
dsc resource test --resource Microsoft.Windows/UserRightsAssignment --input '{"right":"SeDenyBatchLogonRight","identity":["S-1-5-32-546"]}'
```

```yaml
desiredState:
  right: SeDenyBatchLogonRight
  identity:
  - S-1-5-32-546
actualState:
  right: SeDenyBatchLogonRight
  identity: []
  _inDesiredState: false
inDesiredState: false
differingProperties:
- identity
```

To check that an account holds a right regardless of the other holders, add `setAction: add`:

```powershell
dsc resource test --resource Microsoft.Windows/UserRightsAssignment --input '{"right":"SeChangeNotifyPrivilege","identity":["Everyone"],"setAction":"add"}'
```

```yaml
desiredState:
  right: SeChangeNotifyPrivilege
  identity:
  - Everyone
  setAction: add
actualState:
  right: SeChangeNotifyPrivilege
  identity:
  - BUILTIN\Administrators
  - BUILTIN\Backup Operators
  - BUILTIN\Users
  - Everyone
  - NT AUTHORITY\LOCAL SERVICE
  - NT AUTHORITY\NETWORK SERVICE
  _inDesiredState: true
inDesiredState: true
differingProperties:
- identity
```

The resource reports `inDesiredState: true` because `Everyone` holds the right. DSC still lists
`identity` under `differingProperties` because the arrays differ; rely on `inDesiredState` when
you use `add` or `remove`.

## Export every right

To export the holders of every user right, use the [dsc resource export][03] command. The output
is a configuration document with one instance per right, which you can use as the starting point
for a baseline.

```powershell
dsc resource export --resource Microsoft.Windows/UserRightsAssignment
```

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Microsoft.Windows/UserRightsAssignment-0
  type: Microsoft.Windows/UserRightsAssignment
  properties:
    right: SeTrustedCredManAccessPrivilege
    identity: []
- name: Microsoft.Windows/UserRightsAssignment-1
  type: Microsoft.Windows/UserRightsAssignment
  properties:
    right: SeNetworkLogonRight
    identity:
    - BUILTIN\Administrators
    - BUILTIN\Backup Operators
    - BUILTIN\Users
    - Everyone
# Elided for brevity
```

Applying an exported document with `dsc config set` makes every right match the exported holders,
because `setAction` defaults to `replace`.

<!-- Link definitions -->
[01]: ../../../../../../cli/resource/get.md
[02]: ../../../../../../cli/resource/test.md
[03]: ../../../../../../cli/resource/export.md
