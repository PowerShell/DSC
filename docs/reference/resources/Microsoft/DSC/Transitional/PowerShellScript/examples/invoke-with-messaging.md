---
description: >
  Example showing how to emit trace messages from a PowerShellScript resource.
ms.date:     05/10/2026
ms.topic:    reference
title:       Invoke the PowerShellScript resource with trace messaging
---

<!-- markdownlint-disable MD025 -->

# Invoke the PowerShellScript resource with trace messaging

These examples show how you can emit messages from the
[`Microsoft.DSC.Transitional/PowerShellScript` resource][01].

## Emitting errors

By default, any errors raised during script execution cause the execution to emit the error message
and immediately halt script execution. The following example snippets show how you can provide
error details for the user when a script fails.

> [!IMPORTANT]
> The `PowerShellScript` resource runs scripts with `$ErrorActionPreference = 'Stop'` by default.
> Non-terminating errors from cmdlets are treated as terminating errors unless the script or cmdlet
> overrides the error action. Native command failures don't automatically stop script execution;
> script authors should check `$LASTEXITCODE` explicitly unless they enable
> [`$PSNativeCommandUseErrorActionPreference`][12] in PowerShell 7.

### Emitting an error from a failed cmdlet

In this example, the script depends on the `tstoy` command being available on the system. When the
command isn't available, the script fails and reports the error.

```powershell
$instance = @'
getScript: |-
  $tstoyCmd = Get-Command -Name tstoy -CommandType Application |
    Select-Object -ExpandProperty Path

  & $tstoyCmd version --full --format json | ConvertFrom-Json
'@

dsc resource get --resource Microsoft.DSC.Transitional/PowerShellScript --input $instance
```

```Output
<timestamp> ERROR PID <pid>: Exception calling "EndInvoke" with "1" argument(s): "The running command stopped because the preference variable "ErrorActionPreference" or common parameter is set to Stop: The term 'tstoy' is not recognized as a name of a cmdlet, function, script file, or executable program.
Check the spelling of the name, or if a path was included, verify that the path is correct and try again."
<timestamp> ERROR Failed to run process 'pwsh': Command: Resource 'pwsh' [exit code 1] manifest description: PowerShell script execution failed
<timestamp> ERROR Command: Resource 'pwsh' [exit code 1] manifest description: PowerShell script execution failed
```

The first error in the output indicates that the script execution was stopped by the error from the
`Get-Command` invocation, which showed that `tstoy` wasn't available on the system.

### Emitting errors with `Write-Error`

Instead of raising the default error from a failed command, you can use the [`Write-Error`][02]
cmdlet to emit a specific error message. In this example, the script depends on the `tstoy` command
being available on the system. When the command isn't available, the script fails and reports the
error.

```powershell
$instance = @'
getScript: |-
  $tstoyCmd = Get-Command -Name tstoy* -CommandType Application |
      Where-Object {$_.Name -match 'tstoy(\.exe)?' } |
      Select-Object -ExpandProperty Path
  if ([string]::IsNullOrEmpty($tstoyCmd)) {
      Write-Error "command 'tstoy' not found; unable to report version for 'tstoy'"
  }

  & $tstoyCmd version --full --format json | ConvertFrom-Json
'@


dsc resource get --resource Microsoft.DSC.Transitional/PowerShellScript --input $instance
```

```Output
<timestamp> ERROR PID <pid>: Exception calling "EndInvoke" with "1" argument(s): "The running command stopped because the preference variable "ErrorActionPreference" or common parameter is set to Stop: command 'tstoy' not found; unable to report version for 'tstoy'"
<timestamp> ERROR Failed to run process 'pwsh': Command: Resource 'pwsh' [exit code 1] manifest description: PowerShell script execution failed
<timestamp> ERROR Command: Resource 'pwsh' [exit code 1] manifest description: PowerShell script execution failed
```

The first error in the output indicates that the script execution was stopped and includes the
message emitted from the `Write-Error` command.

### Throwing an error from a `catch` block

In the previous error examples, the emitted error includes information about execution stopping
because of the error action preference being set to stop. You can make the error message clearer
by rethrowing the underlying exception from a the `catch` block in a [`try`/`catch` statement][03].

```powershell
$instance = @'
getScript: |-
  try {
      $tstoyCmd = Get-Command -Name tstoy -CommandType Application |
          Select-Object -ExpandProperty Path

      & $tstoyCmd version --full --format json | ConvertFrom-Json
  } catch {
      throw $_.Exception
  }
'@

dsc resource get --resource Microsoft.DSC.Transitional/PowerShellScript --input $instance
```

```Output
<timestamp> ERROR PID <pid>: Exception calling "EndInvoke" with "1" argument(s): "The term 'tstoy' is not recognized as a name of a cmdlet, function, script file, or executable program.
Check the spelling of the name, or if a path was included, verify that the path is correct and try again."
<timestamp> ERROR Failed to run process 'pwsh': Command: Resource 'pwsh' [exit code 1] manifest description: PowerShell script execution failed
<timestamp> ERROR Command: Resource 'pwsh' [exit code 1] manifest description: PowerShell script execution failed
```

## Emitting warning messages

You can emit warning messages from a script with the [`Write-Warning`][04] cmdlet.

This example shows how you can emit a warning from a script without halting execution. The script
looks for the `tstoy` command and returns the version information for that command if it exists. If
the command isn't available, the script raises a warning and returns no output data.

```powershell
$instance = @'
getScript: |-
  $tstoyCmd = Get-Command -Name tstoy* -CommandType Application |
      Where-Object {$_.Name -match 'tstoy(\.exe)?' } |
      Select-Object -ExpandProperty Path

  if ([string]::IsNullOrEmpty($tstoyCmd)) {
      Write-Warning "command 'tstoy' not found; unable to report version for 'tstoy'"
  } else {
      & $tstoyCmd version --full --format json | ConvertFrom-Json
  }
'@


dsc resource get --resource Microsoft.DSC.Transitional/PowerShellScript --input $instance
```

```Output
<timestamp>  WARN PID <pid>: command 'tstoy' not found; unable to report version for 'tstoy'
actualState:
  output: []
```

## Emitting info messages

You can emit `info` level messages for DSC with the [`Write-Verbose`][05] and [`Write-Host`][06]
cmdlets. When a cmdlet used in your script emits verbose or information messages, you can use the
[`-Verbose` common parameter][07] or specify the [`-InformationAction` common parameter][08] as
`Continue` to have those messages emitted for DSC.

### Emitting verbose messages from cmdlets

The following snippet creates a temporary file. It uses commands that emit verbose messages, like
`New-Item`. The example shows how you can specify the `-Verbose` parameter on cmdlets to surface
their verbose messaging in DSC as `info` level trace messages.

```powershell
$instance  = [ordered]@{
    input  = 'create'
    getScript = {
        param(
            [ValidateSet('create', 'delete')]
            [string] $fileOperation
        )

        $tempFolder = "Temp:/dsc/examples/PowerShellScript/messaging"
        $tempFile   = Join-Path $tempFolder 'info.txt'

        if (Test-Path $tempFile) {
            $fileInfo = Get-Item -Path $tempFile 

            [ordered]@{
                path             = $fileInfo.FullName
                exists           = $true
                creationTimeUtc  = $fileInfo.CreationTimeUtc
                lastWriteTimeUtc = $fileInfo.LastWriteTimeUtc
                attributes       = $fileInfo.Attributes.ToString()
            }
        } else {
            [ordered]@{
                path             = $fileInfo.FullName
                exists           = $false
            }
        }
    }.ToString()
    setScript = {
        param(
            [ValidateSet('create', 'delete')]
            [string] $fileOperation
        )

        $tempFolder = "Temp:\dsc\examples\PowerShellScript\messaging"
        $tempFile   = Join-Path $tempFolder 'info.txt'

        switch ($fileOperation) {
            'create' {
                if (-not (Test-Path $tempFolder)) {
                    $null = New-Item -Path $tempFolder -ItemType Directory -Force -Verbose
                }
                if (-not (Test-Path $tempFile)) {
                    $null = New-Item -Path $tempFile -ItemType File -Verbose
                }

                $fileInfo = Get-Item -Path $tempFile 

                [ordered]@{
                    path             = $fileInfo.FullName
                    exists           = $true
                    creationTimeUtc  = $fileInfo.CreationTimeUtc
                    lastWriteTimeUtc = $fileInfo.LastWriteTimeUtc
                    attributes       = $fileInfo.Attributes
                }
            }
            'delete' {
                if (Test-Path $tempFile) {
                    Remove-Item -Path $tempFile -Force -Verbose
                }

                [ordered]@{
                    path             = $fileInfo.FullName
                    exists           = $false
                }
            }
        }
    }.ToString()
}

dsc --trace-level info resource set --resource Microsoft.DSC.Transitional/PowerShellScript --input (
    $instance | ConvertTo-Json -Compress
)
```

```Output
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Performing the operation "Create Directory" on target "Destination: C:\Users\<username>\AppData\Local\Temp\dsc\examples\PowerShellScript".
<timestamp>  INFO PID <pid>: Performing the operation "Create Directory" on target "Destination: C:\Users\<username>\AppData\Local\Temp\dsc\examples\PowerShellScript\messaging".
<timestamp>  INFO PID <pid>: Performing the operation "Create File" on target "Destination: C:\Users\<username>\AppData\Local\Temp\dsc\examples\PowerShellScript\messaging\info.txt".
<timestamp>  INFO diff: key 'creationTimeUtc' missing
<timestamp>  INFO diff: key 'lastWriteTimeUtc' missing
<timestamp>  INFO diff: key 'attributes' missing
<timestamp>  INFO diff: actual array missing expected item
<timestamp>  INFO diff: arrays differ for 'output'
beforeState:
  output:
  - path: null
    exists: false
afterState:
  output:
  - path: C:\Users\<username>\AppData\Local\Temp\dsc\examples\PowerShellScript\messaging\info.txt
    exists: true
    creationTimeUtc: 2026-05-21T17:58:31.2115007Z
    lastWriteTimeUtc: 2026-05-21T17:58:31.2115007Z
    attributes: 32
changedProperties:
- output
```

The info messages emitted by DSC include the verbose messages from creating the temporary directory
and file.

Invoke the resource again but with the `input` set to `delete` to remove the temporary file:

```powershell
$instance.input = 'delete'

dsc --trace-level info resource set --resource Microsoft.DSC.Transitional/PowerShellScript --input (
    $instance | ConvertTo-Json -Compress
)
```

### Emitting verbose messages with `Write-Verbose`

You can surface custom `info` level messages from scripts with the [`Write-Verbose`][05] cmdlet.

The following snippet shows how messages from `Write-Verbose` surface as DSC trace messages.

```powershell
$instance = @'
getScript: |-
  Write-Verbose "Setting things up"
  Write-Verbose "Retrieving data"
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level info resource get @arguments
```

```Output
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <oid>: Setting things up
<timestamp>  INFO PID <oid>: Retrieving data
actualState:
  output: []
```

### Emitting verbose messages with `Write-Host`

You can surface custom `info` level messages from scripts with the [`Write-Host`][06] cmdlet.

The following snippet shows how messages from `Write-Host` surface as DSC trace messages.

```powershell
$instance = @'
getScript: |-
  Write-Host "Setting things up"
  Write-Host "Retrieving data"
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level info resource get @arguments
```

```Output
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Setting things up
<timestamp>  INFO PID <pid>: Retrieving data
actualState:
  output: []
```

## Emitting debug messages

You can emit `debug` level messages for DSC with the [`Write-Debug`][09] cmdlet. When a cmdlet used
in your script emits debug messages, you can use the [`-Debug` common parameter][07] to have those
messages emitted for DSC.

### Emitting debug messages from cmdlets

The following snippet shows how debug messages from commands are captured by the resource. It
defines a function that emits debug messages and then invokes that function.

```powershell
$instance = @'
getScript: |-
  function Get-Data {
    [CmdletBinding()]
    param()

    Write-Debug "Starting process..."
    Write-Debug "Doing things..."
    Write-Debug "Done."
  }

  Get-Data
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level debug resource get @arguments
```

```Output
<timestamp>  INFO dsc_lib::dscresources::command_resource: 69: Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 1218: PID <pid>: Starting process...
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 1218: PID <pid>: Doing things...
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 1218: PID <pid>: Done.
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 850: Process 'pwsh' id <pid> exited with code 0
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 72: Verifying output of get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
actualState:
  output: []
```

The output shows `debug` level messages emitted by the invoked function in the script.

### Emitting debug messages with `Write-Debug`

You can surface custom `debug` level messages from scripts with the [`Write-Debug`][05] cmdlet.

The following snippet shows how messages from `Write-Debug` surface as DSC trace messages.

```powershell
$instance = @'
getScript: |-
  Write-Debug "Setting things up"
  Write-Debug "Retrieving data"
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level debug resource get @arguments
```

```Output
<timestamp>  INFO dsc_lib::dscresources::command_resource: 69: Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 1218: PID <pid>: Setting things up
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 1218: PID <pid>: Retrieving data
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 850: Process 'pwsh' id <pid> exited with code 0
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 72: Verifying output of get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
actualState:
  output: []
```

## Emitting trace messages

You can emit `trace` level messages for DSC with the [`Write-Information`][10] cmdlet. When a
cmdlet used in your script emits debug messages, you can specify the
[`-InformationAction` common parameter][11] as `Continue` to have those messages emitted for DSC.

### Emitting trace messages from cmdlets

The following snippet shows how information messages from commands are captured by the resource as
trace messages. It defines a function that emits information messages and then invokes that
function with `-InformationAction` as `Continue`.

```powershell
$instance = @'
getScript: |-
  function Get-Data {
    [CmdletBinding()]
    param()

    Write-Information "Starting process..."
    Write-Information "Doing things..."
    Write-Information "Done."
  }

  Get-Data -InformationAction Continue
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level trace resource get @arguments
```

```Output
<timestamp>  INFO dsc_lib::dscresources::command_resource: 69: Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> TRACE dsc_lib::dscresources::command_resource: 898: Invoking command 'pwsh' with args Some(["-NoLogo", "-NonInteractive", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "$input | ./psscript.ps1", "get"])
<timestamp> TRACE dsc_lib::dscresources::command_resource: 900: Current working directory: C:\code\dsc\dsc-pr-review\bin\debug
<timestamp> TRACE dsc_lib::dscresources::command_resource: 806: Writing to command STDIN: {"getScript":"function Get-Data {\n  [CmdletBinding()]\n  param()\n\n  Write-Information \"Starting process...\"\n  Write-Information \"Doing things...\"\n  Write-Information \"Done.\"\n}\n\nGet-Data -InformationAction Continue"}
<timestamp> TRACE dsc_lib::dscresources::command_resource: 1220: PID <pid>: Starting process...
<timestamp> TRACE dsc_lib::dscresources::command_resource: 1220: PID <pid>: Doing things...
<timestamp> TRACE dsc_lib::dscresources::command_resource: 1220: PID <pid>: Done.
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 850: Process 'pwsh' id <pid> exited with code 0
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 72: Verifying output of get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> TRACE dsc_lib::dscresources::command_resource: 1083: Verify JSON for 'Microsoft.DSC.Transitional/PowerShellScript': {"output":[]}

actualState:
  output: []
```

The output shows `trace` level messages emitted by the invoked function in the script.

### Emitting trace messages with `Write-Information`

You can surface custom `trace` level messages from scripts with the [`Write-Information`][10] cmdlet.

The following snippet shows how messages from `Write-Information` surface as DSC trace messages.

```powershell
$instance = @'
getScript: |-
  Write-Information "Setting things up"
  Write-Information "Retrieving data"
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level trace resource get @arguments
```

```Output
<timestamp>  INFO dsc_lib::dscresources::command_resource: 69: Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> TRACE dsc_lib::dscresources::command_resource: 898: Invoking command 'pwsh' with args Some(["-NoLogo", "-NonInteractive", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "$input | ./psscript.ps1", "get"])
<timestamp> TRACE dsc_lib::dscresources::command_resource: 900: Current working directory: C:\code\dsc\dsc-pr-review\bin\debug
<timestamp> TRACE dsc_lib::dscresources::command_resource: 806: Writing to command STDIN: {"getScript":"Write-Information \"Setting things up\"\nWrite-Information \"Retrieving data\""}
<timestamp> TRACE dsc_lib::dscresources::command_resource: 1220: PID <pid>: Setting things up
<timestamp> TRACE dsc_lib::dscresources::command_resource: 1220: PID <pid>: Retrieving data
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 850: Process 'pwsh' id <pid> exited with code 0
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 72: Verifying output of get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> TRACE dsc_lib::dscresources::command_resource: 1083: Verify JSON for 'Microsoft.DSC.Transitional/PowerShellScript': {"output":[]}

actualState:
  output: []
```

<!-- Link reference definitions -->
[01]: ../index.md
[02]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-error
[03]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_try_catch_finally
[04]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-warning
[05]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-verbose
[06]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-host
[07]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_commonparameters#-verbose
[08]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_commonparameters#-informationaction
[09]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-debug
[10]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-information
[11]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_commonparameters#-informationaction
[12]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_preference_variables#psnativecommanduseerroractionpreference
