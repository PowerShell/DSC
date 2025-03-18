---
description: >
  Examples showing how you can invoke the Microsoft.Windows/Registry with DSC to create and delete
  a registry key.

ms.date: 03/25/2025
ms.topic: reference
title: Manage a registry key
---

# Manage a registry key

This example shows how you can use the `Microsoft.Windows/Registry` resource to manage whether a
registry key exists. These examples manage the `DscExamples\ManagedKey` registry key in the current
user hive.

## Test whether a key exists

The following snippet shows how you can use the resource with the [dsc resource test][01] command
to check whether the `ManagedKey` registry key exists.

```powershell
$instance = @{
  _exist  = $true
  keyPath = 'HKCU\DscExamples\ManagedKey'
} | ConvertTo-Json

dsc resource test --resource Microsoft.Windows/Registry --input $instance
```

When the registry key doesn't exist, DSC returns the following result:

```yaml
desiredState:
  keyPath: HKCU\DscExamples\ManagedKey
  _exist: true
actualState:
  keyPath: HKCU\DscExamples\ManagedKey
  _exist: false
inDesiredState: false
differingProperties:
- _exist
```

The `inDesiredState` field of the result object is set to `false`, indicating that the
instance isn't in the desired state. The `differingProperties` field indicates that the
`_exist` property is mismatched between the desired state and actual state.

Because the resource uses the [_exist canonical resource property][02], we know that:

- This result indicates that the
registry key doesn't exist on the system.
- The resource will create the instance during a **Set** operation.

## Ensure a registry key exists

To set the system to the desired state and create the registry key, use the [dsc resource set][03]
command.

```powershell
dsc resource set --resource Microsoft.Windows/Registry --input $instance
```

When the resource creates the key, DSC returns the following result:

```yaml
beforeState:
  keyPath: HKCU\DscExamples\ManagedKey
  _exist: false
afterState:
  keyPath: HKCU\DscExamples\ManagedKey
changedProperties:
- _exist
```

You can test the instance again to confirm that the key exists:

```powershell
dsc resource test --resource Microsoft.Windows/Registry --input $instance
```

```yaml
desiredState:
  keyPath: HKCU\DscExamples\ManagedKey
  _exist: true
actualState:
  keyPath: HKCU\DscExamples\ManagedKey
inDesiredState: true
differingProperties: []
```

## Remove a key

The following snippet shows how you can use the [dsc resource delete][04] command to remove the
registry key.

```powershell
dsc resource delete --resource Microsoft.Windows/Registry --input $instance
```

The `dsc resource delete` command doesn't return any output. To verify the registry key no
longer exists, use the `dsc resource get` command.

```powershell
dsc resource get --resource Microsoft.Windows/Registry --input $instance
```

```yaml
actualState:
  keyPath: HKCU\DscExamples\ManagedKey
  _exist: false
```

> [!NOTE]
> Although this example used the **Delete** operation to remove the registry key instance, you can
> also use the **Set** operation.
>
> Because the resource uses the `_exist` canonical property, has the `delete` capability, and
> doesn't have the `setHandlesExist` capability, DSC knows to call the **Delete** operation for the
> resource when an instance defines `_exist` as `false`.

## Cleanup

To return your system to its original state, use the following snippet to remove the `DscExamples`
registry key and any remaining subkeys or values.

```powershell
dsc resource delete --resource Microsoft.Windows/Registry --input @'
keyPath: HKCU\DscExamples
'@
```

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/test.md
[02]: ../../../../../schemas/resource/properties/exist.md
[03]: ../../../../../cli/resource/set.md
[04]: ../../../../../cli/resource/delete.md
