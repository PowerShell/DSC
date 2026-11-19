---
description: >
  Example showing how to use the Microsoft.Windows/AuditPolicyList resource in a DSC configuration
  document to enforce an audit policy baseline with a single instance.
ms.date:     10/19/2026
ms.topic:    reference
title:       Configure an audit policy baseline
---

# Configure an audit policy baseline

This example shows how you can use the `Microsoft.Windows/AuditPolicyList` resource in a DSC
configuration document to enforce the auditing of several audit policy subcategories with a single
instance, as a hardening baseline typically requires.

> [!IMPORTANT]
> All operations of this resource require an elevated (administrator) process context. Run your
> terminal or PowerShell session as Administrator before using `dsc config set`.

## Definition

The configuration document for this example defines one instance of the `AuditPolicyList`
resource that lists three subcategories.

The first entry ensures that both successful and failed logon events are audited. The second
entry ensures that failed credential validation is audited and leaves success auditing as it is,
because the `success` property is omitted. The third entry disables auditing of the
`Other Privilege Use Events` subcategory entirely.

:::code language="yaml" source="auditpolicylist.config.dsc.yaml":::

Copy the configuration document and save it as `auditpolicylist.config.dsc.yaml`.

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
dsc config test --file ./auditpolicylist.config.dsc.yaml
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
  name: Audit policy baseline
  type: Microsoft.Windows/AuditPolicyList
  result:
    desiredState:
      policies:
      - subcategory: Logon
        success: true
        failure: true
      - subcategory: Credential Validation
        failure: true
      - subcategory: Other Privilege Use Events
        success: false
        failure: false
    actualState:
      policies:
      - subcategory: Logon
        success: true
        failure: false
        guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
        category: Logon/Logoff
      - subcategory: Credential Validation
        success: false
        failure: false
        guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
        category: Account Logon
      - subcategory: Other Privilege Use Events
        success: true
        failure: false
        guid: '{0CCE922A-69AE-11D9-BED3-505054503030}'
        category: Privilege Use
    inDesiredState: false
    differingProperties:
    - policies
messages: []
hadErrors: false
```

The instance isn't in the desired state because at least one entry has a flag that differs. DSC
compares only the flags that each entry specifies.

## Set the configuration

To enforce the desired state, use the [dsc config set][02] command.

```powershell
dsc config set --file ./auditpolicylist.config.dsc.yaml
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
  name: Audit policy baseline
  type: Microsoft.Windows/AuditPolicyList
  result:
    beforeState:
      policies:
      - subcategory: Logon
        success: true
        failure: false
        guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
        category: Logon/Logoff
      - subcategory: Credential Validation
        success: false
        failure: false
        guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
        category: Account Logon
      - subcategory: Other Privilege Use Events
        success: true
        failure: false
        guid: '{0CCE922A-69AE-11D9-BED3-505054503030}'
        category: Privilege Use
    afterState:
      policies:
      - subcategory: Logon
        success: true
        failure: true
        guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
        category: Logon/Logoff
      - subcategory: Credential Validation
        success: false
        failure: true
        guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
        category: Account Logon
      - subcategory: Other Privilege Use Events
        success: false
        failure: false
        guid: '{0CCE922A-69AE-11D9-BED3-505054503030}'
        category: Privilege Use
    changedProperties:
    - policies
messages: []
hadErrors: false
```

The resource applied the changes to all three subcategories with a single call to the audit policy
API. The `Credential Validation` entry only changed `failure`; its `success` flag kept the current
value because the configuration document doesn't specify it. Running `dsc config set` again makes
no further changes because the instance is already in the desired state.

## Preview the changes

To see which flags would change without applying them, add the `--what-if` option. The resource
reports the projected state of every entry and describes each change. DSC moves those descriptions
from the resource output to the `whatIf` list in the instance's `metadata`.

```powershell
dsc config set --file ./auditpolicylist.config.dsc.yaml --what-if
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
    - Would change failure auditing for subcategory 'Credential Validation' from false to true
    - Would change success auditing for subcategory 'Other Privilege Use Events' from true to false
  name: Audit policy baseline
  type: Microsoft.Windows/AuditPolicyList
  result:
    beforeState:
      # Elided for brevity: the same as the beforeState of the set operation
    afterState:
      policies:
      - subcategory: Logon
        success: true
        failure: true
        guid: '{0CCE9215-69AE-11D9-BED3-505054503030}'
        category: Logon/Logoff
      - subcategory: Credential Validation
        success: false
        failure: true
        guid: '{0CCE923F-69AE-11D9-BED3-505054503030}'
        category: Account Logon
      - subcategory: Other Privilege Use Events
        success: false
        failure: false
        guid: '{0CCE922A-69AE-11D9-BED3-505054503030}'
        category: Privilege Use
    changedProperties:
    - policies
    - _metadata
messages: []
hadErrors: false
```

<!-- Link definitions -->
[01]: ../../../../../cli/config/test.md
[02]: ../../../../../cli/config/set.md
