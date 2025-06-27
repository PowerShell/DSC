---
description: Demonstrates basic usage of the Microsoft.DSC.Debug/Echo resource
ms.date:     06/22/2025
ms.topic:    reference
title:       Basic echo example
---

This example demonstrates how to use the `Microsoft.DSC.Debug/Echo` to test the output returned by DSC.

## Test the output returned by DSC

The following snippet shows how you can use the resource with the [dsc resource test][01] command to test if the
system is in the desired state.

```powershell
$instance = @{
    output        = 'Hello World!'
} | ConvertTo-Json

dsc resource test --resource Microsoft.DSC.Debug/Echo --input $instance
```

```yaml
desiredState:
  output: Hello World!
actualState:
  output: Hello World!
inDesiredState: true
differingProperties: []
```

> [!NOTE]
> The `Microsoft.DSC.Debug/Echo` resource always returns `inDesiredState: true` because it's a
> test resource designed to echo back values.
> It doesn't actually check or enforce anything on the system - it simply returns whatever value you
> provide as output.

## Using the get capability

The `Microsoft.DSC.Debug/Echo` resource's `get` capability returns the current value in the output property:

```powershell
$instance = @{
    output = 'Hello World!'
} | ConvertTo-Json

dsc resource get --resource Microsoft.DSC.Debug/Echo --input $instance
```

The resource will return the same output value:

```yaml
actualState:
  output: Hello World!
```

## Using the set capability

The `Microsoft.DSC.Debug/Echo` resource's `set` capability simply accepts a value and echoes
it back without modifying anything:

```powershell
$instance = @{
    output = @{
        name = "ExampleSetting"
        value = 123
        enabled = $true
    }
} | ConvertTo-Json

dsc resource set --resource Microsoft.DSC.Debug/Echo --input $instance
```

This will report success and echo the complex object:

```yaml
beforeState:
  output:
    value: 123
    enabled: true
    name: ExampleSetting
afterState:
  output:
    value: 123
    enabled: true
    name: ExampleSetting
changedProperties: []
```

> [!NOTE]
> Even though you're using the `set` capability, no actual changes are made to the system.

## See also

- [Microsoft.DSC.Debug/Echo resource](../index.md)

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/test.md
