---
description: >
    Example showing how to use the Microsoft.Windows/RebootPending resource in a 
    configuration document with an assertion to check for a pending reboot.
ms.date: 03/25/2025
ms.topic: reference
title: Use RebootPending resource in a configuration
---

# Use the RebootPending resource in a configuration

This example demonstrates how to use the `Microsoft.Windows/RebootPending` resource in a configuration document.
The configuration checks if a reboot is pending and, if so, skips the subsequent step using an assertion.

## Definition

This configuration document demonstrates how to use the `Microsoft.Windows/RebootPending` resource together with an assertion.

The first instance defines the desired state for the `ManagedKey` registry key, ensuring it
exists only if no reboot is pending. It uses the `dependsOn` property to reference the assertion resource,
which checks the system's reboot status using the `Microsoft.Windows/RebootPending` resource.
The assertion passes when `rebootPending` is `false`,allowing the registry key resource to run.
If a reboot is pending, the assertion fails and the registry key is not set.

:::code language="yaml" source="pendingReboot.config.dsc.yaml":::

Copy the configuration document and save it as `pendingReboot.config.dsc.yaml`.

## Test configuration

To see whether the system is in the desired state, use the [dsc config test][01] command on the
configuration document.

```powershell
dsc config test --file ./pendingReboot.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: test
    executionType: actual
    startDatetime: 2025-06-03T06:49:22.573486200+02:00
    endDatetime: 2025-06-03T06:49:35.813770500+02:00
    duration: PT13.2402843S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT10.0162818S
  name: Assert pending reboot
  type: Microsoft.DSC/Assertion
  result:
    desiredState:
      $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      resources:
      - name: Check pending reboot
        type: Microsoft.Windows/RebootPending
        properties:
          rebootPending: false
    actualState:
      $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      contentVersion: 1.0.0
      resources:
      - type: Microsoft.Windows/RebootPending
        name: Check pending reboot
        properties:
          rebootPending: false
    inDesiredState: true
    differingProperties: []
- metadata:
    Microsoft.DSC:
      duration: PT0.0549784S
  name: Managed key
  type: Microsoft.Windows/Registry
  result:
    desiredState:
      _exist: true
      keyPath: HKCU\DscExamples\ManagedKey
    actualState:
      keyPath: HKCU\DscExamples\ManagedKey
      _exist: false
    inDesiredState: false
    differingProperties:
    - _exist
messages: []
hadErrors: false
```

Review the individual results to understand whether each instance is in the desired state.

The result for the first instance, named `Check pending reboot`, was:

```yaml
desiredState:
  $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
  resources:
  - name: Check pending reboot
    type: Microsoft.Windows/RebootPending
    properties:
      rebootPending: false
actualState:
  $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
  contentVersion: 1.0.0
  resources:
  - type: Microsoft.Windows/RebootPending
    name: Check pending reboot
    properties:
      rebootPending: false
inDesiredState: true
differingProperties: []
```

The output indicates there is no pending reboot. When you use the **Set** operation on
this confifguration, the second instance will run.

The result for the second instance, named `Managed value`, was:

```yaml
desiredState:
  _exist: true
  keyPath: HKCU\DscExamples\ManagedKey
actualState:
  keyPath: HKCU\DscExamples\ManagedKey
  _exist: false
inDesiredState: false
differingProperties:
- _exist
```

The output indicates the registry path doesn't exist.

The first instance indicates the resource is in the desired state. The second
instance indicates it isn't in the desired state.

## Enforce configuration

To update the system to the desired state, use the [dsc config set][02] command on the configuration document.

```powershell
dsc config set --file ./pendingReboot.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-06-03T06:55:12.123456+02:00
    endDatetime: 2025-06-03T06:55:15.654321+02:00
    duration: PT3.530865S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT2.000000S
  name: Assert pending reboot
  type: Microsoft.DSC/Assertion
  result:
    beforeState:
      $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      resources:
      - name: Check pending reboot
        type: Microsoft.Windows/RebootPending
        properties:
          rebootPending: false
    afterState:
      $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      resources:
      - name: Check pending reboot
        type: Microsoft.Windows/RebootPending
        properties:
          rebootPending: false
    changedProperties: []
- metadata:
    Microsoft.DSC:
      duration: PT0.0549784S
  name: Managed key
  type: Microsoft.Windows/Registry
  result:
    beforeState:
      keyPath: HKCU\DscExamples\ManagedKey
      _exist: false
    afterState:
      keyPath: HKCU\DscExamples\ManagedKey
    changedProperties:
    - _exist
messages: []
hadErrors: false
```

Review the individual results to understand how the resource modified the system to enforce the desired state for each instance.

The result for the assertion instance, named `Assert pending reboot`, was:

```yaml
beforeState:
- name: Check pending reboot
  type: Microsoft.Windows/RebootPending
  result:
    actualState:
      rebootPending: false
afterState:
- metadata:
    Microsoft.DSC:
      duration: PT0.5209322S
  name: Check pending reboot
  type: Microsoft.Windows/RebootPending
  result:
    desiredState:
      rebootPending: false
    actualState:
      rebootPending: false
    inDesiredState: true
    differingProperties: []
changedProperties: []
```

The output indicates the assertion passed and no changes were needed.

The result for the registry key instance, named `Managed key`, was:

```yaml
beforeState:
  keyPath: HKCU\DscExamples\ManagedKey
  _exist: false
afterState:
  keyPath: HKCU\DscExamples\ManagedKey
changedProperties:
- _exist
```

The output indicates that the resource created the registry key.

## Cleanup

To return your system to its original state:

1. Save the following configuration as `registry.cleanup.config.dsc.yaml`.

   :::code language="yaml" source="../../Registry/examples/registry.cleanup.config.dsc.yaml":::

2. Use the **Set** operation on the cleanup configuration document.

   ```powershell
   dsc config set --file ./registry.cleanup.config.dsc.yaml
   ```

<!-- Link reference definitions -->
[01]: ../../../../../cli/config/test.md
[02]: ../../../../../cli/config/set.md