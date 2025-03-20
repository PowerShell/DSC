---
description: >
  Examples showing how you can use the Microsoft.Windows/Registry resource to manage registry keys
  and values in a DSC configuration document.
ms.date: 03/18/2025
ms.topic: reference
title: Configure registry keys and values
---

# Configure registry keys and values

This example shows how you can use the `Microsoft.Windows/Registry` resource to manage registry
keys and values.

## Definition

The configuration document for this example defines two instances of the `Registry` resource.

The first instance defines the desired state for the `ManagedKey` registry key, ensuring it
exists. The second instance defines the desired state for the `ManagedValue` registry value,
ensuring it exists and has the string data `Default`.

:::code language="yaml" source="registry.config.dsc.yaml":::

Copy the configuration document and save it as `registry.config.dsc.yaml`.

## Test the configuration

Use the **Test** operation on the configuration document to see whether the system is in the
desired state.

```powershell
dsc config test --file ./registry.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: test
    executionType: actual
    startDatetime: 2025-03-12T11:51:24.606655-05:00
    endDatetime: 2025-03-12T11:51:25.994942800-05:00
    duration: PT1.3882878S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.2552945S
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
- metadata:
    Microsoft.DSC:
      duration: PT0.0605464S
  name: Managed value
  type: Microsoft.Windows/Registry
  result:
    desiredState:
      _exist: true
      keyPath: HKCU\DscExamples
      valueName: ManagedValue
      valueData:
        String: Default
    actualState:
      keyPath: HKCU\DscExamples
      _exist: false
    inDesiredState: false
    differingProperties:
    - _exist
    - valueName
    - valueData
messages: []
hadErrors: false
```

Review the individual results to understand whether each instance is in the desired state.

The result for the first instance, named `Managed key`, was:

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

The output indicates that the registry key doesn't exist. When you use the **Set** operation on
this configuration, the resource will create the registry key.

The result for the second instance, named `Managed value`, was:

```yaml
desiredState:
  _exist: true
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  valueData:
    String: Default
actualState:
  keyPath: HKCU\DscExamples
  _exist: false
inDesiredState: false
differingProperties:
- _exist
- valueName
- valueData
```

The actual state for the instance reports that `_exist` is `false` and doesn't include the
`valueName` property. For the `Registry` resource, this indicates that the registry key itself
doesn't exist. In this case, the `Registry` resource is reporting that the `DscExamples` registry
key in the current user hive doesn't exist. When the key exists but the value doesn't, the actual
state includes the `valueName` property.

Together, the results show that none of the instances in the configuration are in the desired
state.

## Enforce the configuration

To update the system to the desired state, use the [dsc config set](../../../../../cli/config/set.md)
command on the configuration document.

```powershell
dsc config set --file ./registry.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-03-12T11:59:40.172845800-05:00
    endDatetime: 2025-03-12T11:59:42.127979800-05:00
    duration: PT1.955134S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.6354747S
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
- metadata:
    Microsoft.DSC:
      duration: PT0.2081512S
  name: Managed value
  type: Microsoft.Windows/Registry
  result:
    beforeState:
      keyPath: HKCU\DscExamples
      valueName: ManagedValue
      _exist: false
    afterState:
      keyPath: HKCU\DscExamples
      valueName: ManagedValue
      valueData:
        String: Default
    changedProperties:
    - valueData
    - _exist
messages: []
hadErrors: false
```

Review the individual results to understand how the resource modified the system to enforce the
desired state for each instance.

The result for the first instance, named `Managed key`, was:

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

The result for the second instance, named `Managed value`, was:

```yaml
beforeState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  _exist: false
afterState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  valueData:
    String: Default
changedProperties:
- valueData
- _exist
```

The output indicates that the resource created the registry value with the specified data.

## Cleanup

To return your system to its original state:

1. Save the following configuration as `registry.cleanup.config.dsc.yaml`.

   :::code language="yaml" source="registry.cleanup.config.dsc.yaml":::

2. Use the **Set** operation on the cleanup configuration document.

   ```powershell
   dsc config set --file ./registry.cleanup.config.dsc.yaml
   ```
