---
description: >
  Example showing how you can invoke the Microsoft.DSC.Transitional/RunCommandOnSet resource with DSC to run a simple command.

ms.date: 06/30/2025
ms.topic: reference
title: Run a simple command
---

# Run a simple command

This example shows how you can use the `Microsoft.DSC.Transitional/RunCommandOnSet` resource to
execute a simple command during the set operation.

## Test whether the command would run

The following snippet shows how you can use the resource with the [dsc resource test][01] command to check whether the command would run.

> [!NOTE]
> The `dsc resource test` command performs a synthetic test on this resource. `Microsoft.DSC.Transitional/RunCommandOnSet` doesn't have
> the `test` capability defined in the [resource manifest][02]

```powershell
$instance = @{
  executable = "echo"
  arguments  = @("Configuration applied successfully")
} | ConvertTo-Json

dsc resource test --resource Microsoft.DSC.Transitional/RunCommandOnSet --input $instance
```

When testing the resource, DSC returns a result indicating the desired state:

```yaml
desiredState:
  arguments:
  - Configuration applied successfully
  executable: echo
actualState:
  executable: echo
  arguments:
  - Configuration applied successfully
inDesiredState: true
differingProperties: []
```

The `inDesiredState` field of the result object is set to `true`, indicating that the command would be executed during a set operation.

## Run the command

To execute the command, use the [dsc resource set][03] command.

```powershell
$instance = @{
    executable = "C:\Windows\system32\cmd.exe"
    arguments = @(
        '/C',
        'echo Hello world'
    )
} | ConvertTo-Json
dsc resource set --resource Microsoft.DSC.Transitional/RunCommandOnSet --input $instance
```

When the resource runs the command, DSC returns a result similar to:

```yaml
beforeState:
  executable: cmd
  arguments:
  - /C
  - echo
  - Hello world
afterState:
  executable: cmd
  arguments:
  - /C
  - echo
  - Hello world
changedProperties: []
```

> [!NOTE]
> The output from the command executed by the `runCommandOnSet` resource isn't displayed in the console.
> If you want to capture the output, you should redirect it to a file.

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/test.md
[02]: ../../../../../../schemas/resource/manifest/test.md
[03]: ../../../../../cli/resource/set.md
