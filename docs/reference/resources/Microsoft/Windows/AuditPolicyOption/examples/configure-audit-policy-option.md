---
description: >
  Example showing how to use the Microsoft.Windows/AuditPolicyOption resource in a DSC
  configuration document to enforce the desired state of audit policy options.
ms.date:     10/16/2026
ms.topic:    reference
title:       Configure audit policy options
---

# Configure audit policy options

This example shows how you can use the `Microsoft.Windows/AuditPolicyOption` resource in a DSC
configuration document to enforce the state of audit policy options.

> [!IMPORTANT]
> **Set** operations for this resource require an elevated (administrator) process context. Run
> your terminal or PowerShell session as Administrator before using `dsc config set`.
>
> A change to an audit policy option takes effect after the next restart.

## Definition

The configuration document for this example defines two instances of the `AuditPolicyOption`
resource. The first instance ensures that the use of the Backup and Restore privileges is
audited. The second instance ensures that the system doesn't halt when it can't write audit
records.

:::code language="yaml" source="auditpolicyoption.config.dsc.yaml":::

Copy the configuration document and save it as `auditpolicyoption.config.dsc.yaml`.

## Setup

The output in this example assumes that both options are disabled. You can set the system to this
starting state with the following commands:

```powershell
auditpol /set /option:FullPrivilegeAuditing /value:disable
auditpol /set /option:CrashOnAuditFail /value:disable
```

## Test the configuration

To see whether the system is already in the desired state, use the [dsc config test][01] command.

```powershell
dsc config test --file ./auditpolicyoption.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0312458S
  metadata:
    Microsoft.DSC:
      duration: PT0.0312458S
  name: Audit the use of Backup and Restore privileges
  type: Microsoft.Windows/AuditPolicyOption
  result:
    desiredState:
      name: FullPrivilegeAuditing
      enabled: true
    actualState:
      name: FullPrivilegeAuditing
      enabled: false
    inDesiredState: false
    differingProperties:
    - enabled
- executionInformation:
    duration: PT0.0298731S
  metadata:
    Microsoft.DSC:
      duration: PT0.0298731S
  name: Keep running when audits cannot be logged
  type: Microsoft.Windows/AuditPolicyOption
  result:
    desiredState:
      name: CrashOnAuditFail
      enabled: false
    actualState:
      name: CrashOnAuditFail
      enabled: false
    inDesiredState: true
    differingProperties: []
messages: []
hadErrors: false
```

## Set the configuration

To enforce the desired state, use the [dsc config set][02] command.

```powershell
dsc config set --file ./auditpolicyoption.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0587219S
  metadata:
    Microsoft.DSC:
      duration: PT0.0587219S
  name: Audit the use of Backup and Restore privileges
  type: Microsoft.Windows/AuditPolicyOption
  result:
    beforeState:
      name: FullPrivilegeAuditing
      enabled: false
    afterState:
      name: FullPrivilegeAuditing
      enabled: true
    changedProperties:
    - enabled
- executionInformation:
    duration: PT0.0276594S
  metadata:
    Microsoft.DSC:
      duration: PT0.0276594S
  name: Keep running when audits cannot be logged
  type: Microsoft.Windows/AuditPolicyOption
  result:
    beforeState:
      name: CrashOnAuditFail
      enabled: false
    afterState:
      name: CrashOnAuditFail
      enabled: false
    changedProperties: null
messages:
- name: Audit the use of Backup and Restore privileges
  type: Microsoft.Windows/AuditPolicyOption
  message: A restart is required for the change to audit policy option 'FullPrivilegeAuditing' to take effect
  level: warning
hadErrors: false
```

The first instance changed `enabled` and the resource emitted a warning that the change takes
effect after a restart. The second instance was already in the desired state, so DSC didn't invoke
the **Set** operation for it.

<!-- Link definitions -->
[01]: ../../../../../cli/config/test.md
[02]: ../../../../../cli/config/set.md
