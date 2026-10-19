---
description: >
  Example showing how to use the Microsoft.Windows/AuditPolicy resource in a DSC configuration
  document to enforce the auditing of audit policy subcategories.
ms.date:     10/19/2026
ms.topic:    reference
title:       Configure audit policy
---

# Configure audit policy

This example shows how you can use the `Microsoft.Windows/AuditPolicy` resource in a DSC
configuration document to enforce the auditing of several audit policy subcategories, as a
hardening baseline typically requires.

> [!IMPORTANT]
> All operations of this resource require an elevated (administrator) process context. Run your
> terminal or PowerShell session as Administrator before using `dsc config set`.

## Definition

The configuration document for this example defines three instances of the `AuditPolicy`
resource.

The first instance ensures that both successful and failed logon events are audited. The second
instance ensures that failed credential validation is audited and leaves success auditing as it
is, because the `success` property is omitted. The third instance disables auditing of the
`Other Privilege Use Events` subcategory entirely.

:::code language="yaml" source="auditpolicy.config.dsc.yaml":::

Copy the configuration document and save it as `auditpolicy.config.dsc.yaml`.

> [!TIP]
> A baseline usually sets many subcategories. Instead of one `AuditPolicy` instance per
> subcategory, you can list them all in a single [Microsoft.Windows/AuditPolicyList][03] instance,
> as shown in [Configure an audit policy baseline][04].

## Setup

The output in this example assumes that the system audits successful logons only, doesn't audit
credential validation, and audits successful privilege use events. You can set the system to this
starting state with the following commands:

```powershell
auditpol /set /subcategory:"Logon" /success:enable /failure:disable
auditpol /set /subcategory:"Credential Validation" /success:disable /failure:disable
auditpol /set /subcategory:"Other Privilege Use Events" /success:enable /failure:disable
```

## Test the configuration

To see whether the system is already in the desired state, use the [dsc config test][01] command.

```powershell
dsc config test --file ./auditpolicy.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0411372S
  metadata:
    Microsoft.DSC:
      duration: PT0.0411372S
  name: Audit successful and failed logons
  type: Microsoft.Windows/AuditPolicy
  result:
    desiredState:
      subcategory: Logon
      success: true
      failure: true
    actualState:
      subcategory: Logon
      success: true
      failure: false
      guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
      category: Logon/Logoff
    inDesiredState: false
    differingProperties:
    - failure
- executionInformation:
    duration: PT0.0398114S
  metadata:
    Microsoft.DSC:
      duration: PT0.0398114S
  name: Audit failed credential validation
  type: Microsoft.Windows/AuditPolicy
  result:
    desiredState:
      subcategory: Credential Validation
      failure: true
    actualState:
      subcategory: Credential Validation
      success: false
      failure: false
      guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
      category: Account Logon
    inDesiredState: false
    differingProperties:
    - failure
- executionInformation:
    duration: PT0.0402517S
  metadata:
    Microsoft.DSC:
      duration: PT0.0402517S
  name: Disable auditing of other privilege use events
  type: Microsoft.Windows/AuditPolicy
  result:
    desiredState:
      subcategory: Other Privilege Use Events
      success: false
      failure: false
    actualState:
      subcategory: Other Privilege Use Events
      success: true
      failure: false
      guid: '{0CCE922A-69AE-11D9-BED3-505054503030}'
      category: Privilege Use
    inDesiredState: false
    differingProperties:
    - success
messages: []
hadErrors: false
```

Each instance reports `inDesiredState: false` and lists only the flags that differ. The second
instance doesn't list `success` because the configuration document doesn't specify it.

## Set the configuration

To enforce the desired state, use the [dsc config set][02] command.

```powershell
dsc config set --file ./auditpolicy.config.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT0.0824631S
  metadata:
    Microsoft.DSC:
      duration: PT0.0824631S
  name: Audit successful and failed logons
  type: Microsoft.Windows/AuditPolicy
  result:
    beforeState:
      subcategory: Logon
      success: true
      failure: false
      guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
      category: Logon/Logoff
    afterState:
      subcategory: Logon
      success: true
      failure: true
      guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
      category: Logon/Logoff
    changedProperties:
    - failure
- executionInformation:
    duration: PT0.0811207S
  metadata:
    Microsoft.DSC:
      duration: PT0.0811207S
  name: Audit failed credential validation
  type: Microsoft.Windows/AuditPolicy
  result:
    beforeState:
      subcategory: Credential Validation
      success: false
      failure: false
      guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
      category: Account Logon
    afterState:
      subcategory: Credential Validation
      success: false
      failure: true
      guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
      category: Account Logon
    changedProperties:
    - failure
- executionInformation:
    duration: PT0.0807443S
  metadata:
    Microsoft.DSC:
      duration: PT0.0807443S
  name: Disable auditing of other privilege use events
  type: Microsoft.Windows/AuditPolicy
  result:
    beforeState:
      subcategory: Other Privilege Use Events
      success: true
      failure: false
      guid: '{0CCE922A-69AE-11D9-BED3-505054503030}'
      category: Privilege Use
    afterState:
      subcategory: Other Privilege Use Events
      success: false
      failure: false
      guid: '{0CCE922A-69AE-11D9-BED3-505054503030}'
      category: Privilege Use
    changedProperties:
    - success
messages: []
hadErrors: false
```

The second instance only changed `failure`; its `success` flag kept the current value because the
configuration document doesn't specify it. Running `dsc config set` again makes no further
changes because every instance is already in the desired state.

## Preview the changes

To see which flags would change without applying them, add the `--what-if` option to the
`dsc config set` command. The resource reports the projected state and describes each change.
DSC moves those descriptions from the resource output to the `whatIf` list in the instance's
`metadata`.

```powershell
dsc config set --file ./auditpolicy.config.dsc.yaml --what-if
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
    - Would change failure auditing for subcategory 'Logon' from false to true
  name: Audit successful and failed logons
  type: Microsoft.Windows/AuditPolicy
  result:
    beforeState:
      subcategory: Logon
      success: true
      failure: false
      guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
      category: Logon/Logoff
    afterState:
      subcategory: Logon
      success: true
      failure: true
      guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
      category: Logon/Logoff
    changedProperties:
    - failure
    - _metadata
# Elided for brevity
```

<!-- Link definitions -->
[01]: ../../../../../../cli/config/test.md
[02]: ../../../../../../cli/config/set.md
[03]: ../../AuditPolicyList/index.md
[04]: ../../AuditPolicyList/examples/configure-audit-policy-list.md
