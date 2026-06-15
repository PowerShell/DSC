---
description: >
  Example showing how to use the PowerShellScript resource in a DSC configuration
  document.
ms.date:     05/10/2026
ms.topic:    reference
title:       Configure a system with the PowerShellScript resource
---

<!-- markdownlint-disable MD025 -->

# Configure a system with the PowerShellScript resource

This example shows how you can use the [`Microsoft.DSC.Transitional/PowerShellScript`][01] resource
in a configuration both to invoke non-idempotent scripts and to idempotently manage a message of
the day file that doesn't have a specific DSC resource.

## Definition

The configuration document for this example defines two instances of the resource:

1. The first instance, `Report processor info`, returns the number of processor cores and the
   processor architecture from both `getScript` and `setScript`. This instance is informational
   only - it doesn't modify the system.
1. The second instance, `Message of the Day`, idempotently manages a message of the day file. It
   uses `input` to define the contents of the file and pulls the value for the input from the
   `parameters` definition. It defines all three script properties: `getScript` to return the
   actual state, `testScript` to determine if the instance is in the desired state, and `setScript`
   to enforce the desired state.

   The `getScript` and `setScript` definitions return the same structured output representing the
   state of the MOTD file to make monitoring how the instance modifies the system easier. All three
   script definitions use the `Write-Verbose` cmdlet to emit informational messages about what the
   instance is doing. In particular the messages from `testScript` describe whether and how the
   file isn't in the desired state to address the limited information the script can surface in its
   output.

:::code language="yaml" source="psscript.config.dsc.yaml":::

Copy the configuration document and save it as `psscript.config.dsc.yaml`.

## Get the current state

To retrieve the current state of the system, use the [dsc config get][02] command on the
configuration document.

```powershell
dsc --trace-level info config get --file ./psscript.config.dsc.yaml
```

```Messages
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Checking for MOTD file at 'Temp:/example.motd'
<timestamp>  INFO PID <pid>: MOTD file not found at 'Temp:/example.motd
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  Microsoft.DSC:
    # Elided for brevity
results:
- executionInformation:
    duration: PT1.2985379S
  metadata:
    Microsoft.DSC:
      duration: PT1.2985379S
  name: Report processor info
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    actualState:
      output:
      - processorCount: 8
        processorArchitecture: X64
- executionInformation:
    duration: PT0.9556133S
  metadata:
    Microsoft.DSC:
      duration: PT0.9556133S
  name: Message of the Day
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    actualState:
      output:
      - filePath: Temp:/example.motd
        exists: false
messages: []
hadErrors: false
```

The command emitted messages to stderr and the result to stdout. The messages include informational
messages from `getScript` for the message of the day instance indicating that the script looked for
but did not find the MOTD file.

The result includes structured output from both instances:

- The processor report instance shows that the system has `8` cores and is an `X64` architecture.
- The message of the day instance shows that the expected MOTD file doesn't exist at
  `Temp:/example.motd`.

## Enforce the desired state

To update the system to the desired state, use the [dsc config set][03] command on the
configuration document.

```powershell
dsc --trace-level info config set --file ./psscript.config.dsc.yaml
```

```Messages
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Checking for MOTD file at 'Temp:/example.motd'
<timestamp>  INFO PID <pid>: MOTD file not found at 'Temp:/example.motd'
<timestamp>  INFO PID <pid>: MOTD file not found at 'Temp:/example.motd', creating new file
<timestamp>  INFO PID <pid>: MOTD file created at 'Temp:/example.motd', setting content
<timestamp>  INFO diff: key 'motd' missing
<timestamp>  INFO diff: key 'lastUpdated' missing
<timestamp>  INFO diff: actual array missing expected item
<timestamp>  INFO diff: arrays differ for 'output
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  Microsoft.DSC:
    # Elided for brevity
results:
- executionInformation:
    duration: PT2.1871641S
  metadata:
    Microsoft.DSC:
      duration: PT2.1871641S
  name: Report processor info
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    beforeState:
      output:
      - processorCount: 8
        processorArchitecture: X64
    afterState:
      output:
      - processorCount: 8
        processorArchitecture: X64
    changedProperties: []
- executionInformation:
    duration: PT1.708226S
  metadata:
    Microsoft.DSC:
      duration: PT1.708226S
  name: Message of the Day
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    beforeState:
      output:
      - exists: false
        filePath: Temp:/example.motd
    afterState:
      output:
      - exists: true
        motd: Hello, friend!
        filePath: Temp:/example.motd
        lastUpdated: 2026-06-02T18:05:16.8811712-05:00
    changedProperties:
    - output
messages: []
hadErrors: false
```

As before, the message of the day instance surfaces informational messages. The messages show that
the MOTD file wasn't found and then the `setScript` reports that it is creating the file and
setting the content.

It's easier to review the result data for each instance separately:

- ```yaml
  name: Report processor info
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    beforeState:
      output:
      - processorCount: 8
        processorArchitecture: X64
    afterState:
      output:
      - processorCount: 8
        processorArchitecture: X64
    changedProperties: []
  ```

  The processor info report shows the same state for the system before and after the **Set**
  operation. If the instance didn't define `setScript` then `afterState` would be an empty object
  (`{}`) and the `changedProperties` field would report that `output` was modified. Providing
  identical output for the `setScript` ensures that the result doesn't imply any system changes.

- ```yaml
  name: Message of the Day
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    beforeState:
      output:
      - exists: false
        filePath: Temp:/example.motd
    afterState:
      output:
      - exists: true
        motd: Hello, friend!
        filePath: Temp:/example.motd
        lastUpdated: 2026-06-02T18:05:16.8811712-05:00
    changedProperties:
    - output
  ```

  The result for the message of the day instance shows that `exists` changed from `false` to `true`.
  The `afterState` also includes the `motd` property showing the newly-set MOTD and reports the
  last updated time for the file.

If you invoke the **Set** operation for the configuration again you should see that neither instance
modifies the system:

```powershell
dsc --trace-level info config set --file ./psscript.config.dsc.yaml
```

```Messages
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Checking for MOTD file at 'Temp:/example.motd'
<timestamp>  INFO PID <pid>: MOTD file found at 'Temp:/example.motd', retrieving content and last updated time
<timestamp>  INFO PID <pid>: MOTD file found at 'Temp:/example.motd', checking content
<timestamp>  INFO PID <pid>: MOTD content matches desired value, no update needed
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  Microsoft.DSC:
    # Elided for brevity
results:
- executionInformation:
    duration: PT3.8028321S
  metadata:
    Microsoft.DSC:
      duration: PT3.8028321S
  name: Report processor info
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    beforeState:
      output:
      - processorCount: 8
        processorArchitecture: X64
    afterState:
      output:
      - processorCount: 8
        processorArchitecture: X64
    changedProperties: []
- executionInformation:
    duration: PT2.6216447S
  metadata:
    Microsoft.DSC:
      duration: PT2.6216447S
  name: Message of the Day
  type: Microsoft.DSC.Transitional/PowerShellScript
  result:
    beforeState:
      output:
      - filePath: Temp:/example.motd
        motd: Hello, friend!
        exists: true
        lastUpdated: 2026-06-03T08:46:38.0491245-05:00
    afterState:
      output:
      - motd: Hello, friend!
        exists: true
        lastUpdated: 2026-06-03T08:46:38.0491245-05:00
        filePath: Temp:/example.motd
    changedProperties: []
messages: []
hadErrors: false
```

## Cleanup

To return your system to its original state, invoke the following PowerShell command to remove the
MOTD file from the `Temp:/` folder:

```powershell
Remove-Item -Path 'Temp:/example.motd' -Verbose
```

<!-- Link reference definitions -->
[01]: ../index.md
[02]: ../../../../../../cli/config/get.md
[03]: ../../../../../../cli/config/set.md
