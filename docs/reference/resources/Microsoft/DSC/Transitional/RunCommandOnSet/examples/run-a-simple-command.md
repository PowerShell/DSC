---
description: >
  Example showing how you can invoke the Microsoft.DSC.Transitional/RunCommandOnSet resource with DSC to run a simple command.

ms.date: 06/30/2025
ms.topic: reference
title: Run a simple command
---

# Run a simple command

This example shows how you can use the `Microsoft.DSC.Transitional/RunCommandOnSet` resource to
execute a simple command during the **Set** operation.

## Test whether the command would run

The following snippet shows how you can use the resource with the [dsc resource test][00] command to check whether
the command would run.

> [!NOTE]
> The `dsc resource test` command performs a synthetic test on this resource. `Microsoft.DSC.Transitional/RunCommandOnSet` doesn't have
> the `test` capability defined in the [resource manifest][01]

```powershell
$instance = @{
    executable = "C:\Windows\system32\cmd.exe"
    arguments = @(
        '/C',
        'echo Hello world'
    )
} | ConvertTo-Json

dsc resource test --resource Microsoft.DSC.Transitional/RunCommandOnSet --input $instance
```

When testing the resource, DSC returns a result indicating the desired state:

```yaml
desiredState:
  arguments:
  - /C
  - echo Hello world
  executable: C:\Windows\system32\cmd.exe
actualState:
  executable: C:\Windows\system32\cmd.exe
  arguments:
  - /C
  - echo Hello world
inDesiredState: true
differingProperties: []
```

The `inDesiredState` field always returns `true` because of [pretest][02] is supported.
This means the command is always executed during the **Set** operation.

## Run the command

To execute the command, use the [dsc resource set][03] command.

```powershell
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
[00]: ../../../../../cli/resource/test.md
[01]: ../../../../../../schemas/resource/manifest/test.md
[02]: ../../../../../../../reference/cli/resource/set.md
[03]: ../../../../../cli/resource/set.md
