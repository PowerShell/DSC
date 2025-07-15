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

## Run the command

The following snippet shows how you can invoke the resource to execute a custom command with [dsc resource set][00].

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
[00]: ../../../../../cli/resource/set.md
