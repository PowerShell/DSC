---
description: >
  Example showing how to use the Microsoft.Windows/UserRightsAssignment resource in a DSC
  configuration document to enforce user rights assignments.
ms.date:     10/19/2026
ms.topic:    reference
title:       Configure user rights assignments
---

# Configure user rights assignments

This example shows how you can use the `Microsoft.Windows/UserRightsAssignment` resource in a DSC
configuration document to enforce user rights the way a hardening baseline requires, using each of
the three `setAction` modes.

> [!IMPORTANT]
> All operations of this resource require an elevated (administrator) process context. Run your
> terminal or PowerShell session as Administrator before using `dsc config set`.

## Definition

The configuration document for this example defines three instances of the
`UserRightsAssignment` resource.

The first instance makes `BUILTIN\Guests` the only account that is denied batch logon; because
`setAction` isn't specified, it defaults to `replace`. The second instance ensures that
`NT SERVICE\ALL SERVICES` can log on as a service without touching the other holders. The third
instance removes the direct grant of `SeInteractiveLogonRight` from `BUILTIN\Guests` and keeps
the other holders.

> [!NOTE]
> `setAction: remove` revokes only the direct assignment. Windows grants a right to a user when
> any group the user belongs to holds it, so a member of `Guests` can still log on locally through
> another group that holds `SeInteractiveLogonRight`. To make sure that guests can't log on
> locally regardless of their other group memberships, add `BUILTIN\Guests` to
> `SeDenyInteractiveLogonRight` instead; a deny right always takes precedence over the matching
> allow right.

:::code language="yaml" source="userrightsassignment.config.dsc.yaml":::

Copy the configuration document and save it as `userrightsassignment.config.dsc.yaml`.

## Setup

The output in this example assumes that nobody is denied batch logon, that `ALL SERVICES` already
holds `SeServiceLogonRight`, and that `Guests` holds `SeInteractiveLogonRight`. You can set the
system to this starting state with a security template:

```powershell
@'
[Unicode]
Unicode=yes
[Privilege Rights]
SeDenyBatchLogonRight =
SeInteractiveLogonRight = *S-1-5-32-544,*S-1-5-32-545,*S-1-5-32-546,*S-1-5-32-551
[Version]
signature="$CHICAGO$"
Revision=1
'@ | Set-Content -Path "$env:TEMP\setup.inf" -Encoding unicode
secedit /configure /db "$env:TEMP\setup.sdb" /cfg "$env:TEMP\setup.inf" /areas USER_RIGHTS /quiet
```

## Test the configuration

To see whether the system is already in the desired state, use the [dsc config test][01] command.

```powershell
dsc config test --file ./userrightsassignment.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0532118S
  metadata:
    Microsoft.DSC:
      duration: PT0.0532118S
  name: Deny batch logon to guests only
  type: Microsoft.Windows/UserRightsAssignment
  result:
    desiredState:
      right: SeDenyBatchLogonRight
      identity:
      - BUILTIN\Guests
    actualState:
      right: SeDenyBatchLogonRight
      identity: []
      _inDesiredState: false
    inDesiredState: false
    differingProperties:
    - identity
- executionInformation:
    duration: PT0.0498742S
  metadata:
    Microsoft.DSC:
      duration: PT0.0498742S
  name: Services may log on as a service
  type: Microsoft.Windows/UserRightsAssignment
  result:
    desiredState:
      right: SeServiceLogonRight
      identity:
      - NT SERVICE\ALL SERVICES
      setAction: add
    actualState:
      right: SeServiceLogonRight
      identity:
      - NT SERVICE\ALL SERVICES
      - NT VIRTUAL MACHINE\Virtual Machines
      _inDesiredState: true
    inDesiredState: true
    differingProperties:
    - identity
- executionInformation:
    duration: PT0.0511306S
  metadata:
    Microsoft.DSC:
      duration: PT0.0511306S
  name: Remove the direct local logon grant from Guests
  type: Microsoft.Windows/UserRightsAssignment
  result:
    desiredState:
      right: SeInteractiveLogonRight
      identity:
      - BUILTIN\Guests
      setAction: remove
    actualState:
      right: SeInteractiveLogonRight
      identity:
      - BUILTIN\Administrators
      - BUILTIN\Backup Operators
      - BUILTIN\Guests
      - BUILTIN\Users
      _inDesiredState: false
    inDesiredState: false
    differingProperties:
    - identity
messages: []
hadErrors: false
```

The second instance is in the desired state because `ALL SERVICES` already holds the right. The
resource decides `inDesiredState` from `setAction`; DSC still lists `identity` under
`differingProperties` because the arrays differ.

## Set the configuration

To enforce the desired state, use the [dsc config set][02] command.

```powershell
dsc config set --file ./userrightsassignment.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0987421S
  metadata:
    Microsoft.DSC:
      duration: PT0.0987421S
  name: Deny batch logon to guests only
  type: Microsoft.Windows/UserRightsAssignment
  result:
    beforeState:
      right: SeDenyBatchLogonRight
      identity: []
    afterState:
      right: SeDenyBatchLogonRight
      identity:
      - BUILTIN\Guests
    changedProperties:
    - identity
- executionInformation:
    duration: PT0.0509873S
  metadata:
    Microsoft.DSC:
      duration: PT0.0509873S
  name: Services may log on as a service
  type: Microsoft.Windows/UserRightsAssignment
  result:
    beforeState:
      right: SeServiceLogonRight
      identity:
      - NT SERVICE\ALL SERVICES
      setAction: add
    afterState:
      right: SeServiceLogonRight
      identity:
      - NT SERVICE\ALL SERVICES
      - NT VIRTUAL MACHINE\Virtual Machines
      _inDesiredState: true
    changedProperties: null
- executionInformation:
    duration: PT0.0975219S
  metadata:
    Microsoft.DSC:
      duration: PT0.0975219S
  name: Remove the direct local logon grant from Guests
  type: Microsoft.Windows/UserRightsAssignment
  result:
    beforeState:
      right: SeInteractiveLogonRight
      identity:
      - BUILTIN\Administrators
      - BUILTIN\Backup Operators
      - BUILTIN\Guests
      - BUILTIN\Users
    afterState:
      right: SeInteractiveLogonRight
      identity:
      - BUILTIN\Administrators
      - BUILTIN\Backup Operators
      - BUILTIN\Users
    changedProperties:
    - identity
messages: []
hadErrors: false
```

DSC skipped the second instance because its test reported it in the desired state, so DSC reports
the desired state as `beforeState` and the tested state as `afterState`. The third instance only
revoked the direct grant from `Guests` and kept the other holders, because it uses
`setAction: remove`.

## Preview the changes

To see which accounts would be granted or revoked without applying the change, add the
`--what-if` option. The resource projects the holders each instance would have under its own
`setAction` and describes each change. DSC moves those descriptions from the resource output to
the `whatIf` list in the instance's `metadata`.

```powershell
dsc config set --file ./userrightsassignment.config.dsc.yaml --what-if
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    # Elided for brevity
  metadata:
    Microsoft.DSC:
      # Elided for brevity
    whatIf:
    - Would grant 'SeDenyBatchLogonRight' (Deny log on as a batch job) to 'BUILTIN\Guests'
  name: Deny batch logon to guests only
  type: Microsoft.Windows/UserRightsAssignment
  result:
    beforeState:
      right: SeDenyBatchLogonRight
      identity: []
    afterState:
      right: SeDenyBatchLogonRight
      identity:
      - BUILTIN\Guests
    changedProperties:
    - identity
    - _metadata
- executionInformation:
    # Elided for brevity
  metadata:
    Microsoft.DSC:
      # Elided for brevity
  name: Services may log on as a service
  type: Microsoft.Windows/UserRightsAssignment
  result:
    beforeState:
      right: SeServiceLogonRight
      identity:
      - NT SERVICE\ALL SERVICES
      - NT VIRTUAL MACHINE\Virtual Machines
    afterState:
      right: SeServiceLogonRight
      identity:
      - NT SERVICE\ALL SERVICES
      - NT VIRTUAL MACHINE\Virtual Machines
    changedProperties: []
- executionInformation:
    # Elided for brevity
  metadata:
    Microsoft.DSC:
      # Elided for brevity
    whatIf:
    - Would revoke 'SeInteractiveLogonRight' (Allow log on locally) from 'BUILTIN\Guests'
  name: Remove the direct local logon grant from Guests
  type: Microsoft.Windows/UserRightsAssignment
  result:
    beforeState:
      right: SeInteractiveLogonRight
      identity:
      - BUILTIN\Administrators
      - BUILTIN\Backup Operators
      - BUILTIN\Guests
      - BUILTIN\Users
    afterState:
      right: SeInteractiveLogonRight
      identity:
      - BUILTIN\Administrators
      - BUILTIN\Backup Operators
      - BUILTIN\Users
    changedProperties:
    - identity
    - _metadata
messages: []
hadErrors: false
```

The first instance uses the default `replace`, so the preview lists `Guests` as the only holder.
The second instance uses `add` and already holds the right, so the preview keeps both holders and
describes no change. The third instance uses `remove`, so the preview revokes only `Guests` and
keeps the other holders.

<!-- Link definitions -->
[01]: ../../../../../../cli/config/test.md
[02]: ../../../../../../cli/config/set.md
