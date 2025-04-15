---
description: >
  Examples showing how you can invoke the Microsoft.Windows/Registry resource with DSC to create, modify, and delete
  a registry key value.

ms.date: 03/25/2025
ms.topic: reference
title: Manage a registry value
---

# Manage a registry value

This example shows how you can use the `Microsoft.Windows/Registry` resource to manage whether a
registry value exists and its data. These examples manage the `ManagedValue` registry value for the
`DscExamples` registry key in the current user hive.

## Test whether a value exists

The following snippet shows how you can use the resource with the [dsc resource test][01] command
to check whether the `ManagedValue` registry value exists.

```powershell
$instance   = @{
  _exist    = $true
  keyPath   = 'HKCU\DscExamples'
  valueName = 'ManagedValue'
}
$instanceJson = $instance | ConvertTo-Json

dsc resource test --resource Microsoft.Windows/Registry --input $instanceJson
```

When the registry key doesn't exist, DSC returns the following result:

```yaml
desiredState:
  _exist: true
  valueName: ManagedValue
  keyPath: HKCU\DscExamples
actualState:
  keyPath: HKCU\DscExamples
  _exist: false
inDesiredState: false
differingProperties:
- _exist
- valueName
```

The `inDesiredState` field of the result object is set to `false`, indicating that the
instance isn't in the desired state. The `differingProperties` field indicates that the
`_exist` property is mismatched between the desired state and actual state.

In this case, the `valueName` property isn't returned from the resource. When the actual state of a
`Registry` resource instance doesn't specify the `valueName` property and `_exist` is false, it
indicates that the _key_ doesn't exist.

Because the resource uses the [_exist canonical resource property][02], we know that:

- This result indicates that the
registry key doesn't exist on the system.
- The resource will create the instance during a **Set** operation.

To show the difference, first use the following snippet to create the registry key with the
[dsc resource set][03] command.

```powershell
dsc resource set --resource Microsoft.Windows/Registry --input @'
keyPath: HKCU\DscExamples
_exist: true
'@
```

Then test the instance again:

```powershell
dsc resource test --resource Microsoft.Windows/Registry --input $instanceJson
```

```yaml
desiredState:
  _exist: true
  valueName: ManagedValue
  keyPath: HKCU\DscExamples
actualState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  _exist: false
inDesiredState: false
differingProperties:
- _exist
```

Now that the `DscExamples` registry key exists, the `valueName` property is included in the actual
state and `_exist` is defined as `false` - the key exists but the value doesn't.

## Ensure a registry value exists

In the previous section, the desired state of the instance tested for whether the `ManagedValue`
registry value existed. However, the Registry API requires _creating_ a registry value to define
data for the value. Before you can use the **Set** operation to create the value, the desired state
must define the `valueData` property.

The following snippet defines the data for the registry value as the string `Default` and updates
the `$instanceJson` variable to represent the updated desired state.

```powershell
$instance.valueData = @{ String = 'Default' }
$instance | ConvertTo-Json -OutVariable instanceJson
```

```json
{
  "_exist": true,
  "valueName": "ManagedValue",
  "valueData": {
    "String": "Default"
  },
  "keyPath": "HKCU\\DSC\\Examples"
}
```

Next, invoke the **Set** operation for the instance.

```powershell
dsc resource set --resource Microsoft.Windows/Registry --input $instanceJson
```

When the resource creates the value, DSC returns the following result:

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

You can test the instance to confirm that the key exists:

```powershell
dsc resource test --resource Microsoft.Windows/Registry --input $instanceJson
```

```yaml
desiredState:
  _exist: true
  valueName: ManagedValue
  valueData:
    String: Default
  keyPath: HKCU\DscExamples
actualState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  valueData:
    String: Default
inDesiredState: true
differingProperties: []
```

## Modify the data for a registry value

The previous section created the `ManagedValue` registry value and defined the value data as the
string `Default`. The following snippet shows how you can modify the data with the **Set**
operation.

First, change the desired state for the instance to specify the value as `Modified` and update the
`$instanceJson` variable to reflect the new desired state.

```powershell
$instance.valueData.String = 'Modified'
$instance | ConvertTo-Json -OutVariable instanceJson
```

```json
{
  "_exist": true,
  "valueName": "ManagedValue",
  "valueData": {
    "String": "Modified"
  },
  "keyPath": "HKCU\\DSC\\Examples"
}
```

Next, compare the actual state of the instance to the desired state with the **Test** operation.

```powershell
dsc resource test --resource Microsoft.Windows/Registry --input $instanceJson
```

```yaml
desiredState:
  _exist: true
  valueName: ManagedValue
  valueData:
    String: Modified
  keyPath: HKCU\DscExamples
actualState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  valueData:
    String: Default
inDesiredState: false
differingProperties:
- valueData
```

As expected, the output shows that the `valueData` isn't in the desired state.

Finally, invoke the **Set** operation.

```powershell
dsc resource set --resource Microsoft.Windows/Registry --input $instanceJson
```

```yaml
beforeState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  valueData:
    String: Default
afterState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  valueData:
    String: Modified
changedProperties:
- valueData
```

The resource reports that it changed the `valueData` property and shows the actual state after the
**Set** operation as `Modified` instead of `Default`.

## Remove a registry value

The following snippet shows how you can use the [dsc resource delete][04] command to remove the
`ManagedValue` registry value.

```powershell
dsc resource delete --resource Microsoft.Windows/Registry --input $instanceJson
```

The `dsc resource delete` command doesn't return any output. To verify the registry value no
longer exists, use the `dsc resource get` command.

```powershell
dsc resource get --resource Microsoft.Windows/Registry --input $instanceJson
```

```yaml
actualState:
  keyPath: HKCU\DscExamples
  valueName: ManagedValue
  _exist: false
```

> [!NOTE]
> Although this example used the **Delete** operation to remove the registry value instance, you can
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
