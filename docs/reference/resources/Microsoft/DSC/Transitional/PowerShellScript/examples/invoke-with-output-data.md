---
description: >
  Example showing how to return output data from a PowerShellScript resource.
ms.date:     05/10/2026
ms.topic:    reference
title:       Invoke the PowerShellScript resource with output data
---

<!-- markdownlint-disable MD025 -->

# Invoke the PowerShellScript resource with output data

These examples show how you can return output from the
[`Microsoft.DSC.Transitional/PowerShellScript` resource][01].

## Output data types

All output that a script emits for this resource is inserted into the `output` array for the
resource instance. The resource uses the `ConvertTo-Json` cmdlet for every item emitted to the
[Success stream][02]. The converted representation is what the resource inserts into the `output`
array.

When the resource serializes the output data as JSON it retains up to `9` levels of depth. This can
make the output for typical PowerShell objects a script may return very large and difficult to
parse in the result for an operation.

### Outputting scalar values

The following snippet shows how scalar values (not objects or arrays) are handled by the resource
when emitted by a script. Scalar values include strings, integers, floats, booleans, and `$null`.

```powershell
$instance = @'
getScript: |-
   $true   # boolean scalar value
   1       # integer scalar value
   1.2     # float scalar value
   $null   # null scalar value
   'apple' # string scalar value
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc resource get @arguments
```

```yaml
actualState:
  output:
  - true
  - 1
  - 1.2
  - null
  - apple
```

### Outputting objects

When a script emits objects that aren't scalar values, the conversion to JSON representation
includes up to `9` levels of depth. Objects often have properties that are _also_ objects with
sub-properties or arrays of nested objects.

When the object output is particularly large and complex it can cause the resource operation to
fail when DSC needs to validate the output data. The following snippet shows how emitting a
`[FileInfo]` object directly can cause the resource to fail.

The script creates a new temporary file, which emits the `[FileInfo]` object for the new file as
output.

```powershell
$instance = @'
getScript: |-
   $filePath = 'Temp:/dsc/examples/PowerShellScript/output.txt'

   New-Item -Path $filePath -Force
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level debug resource get @arguments
```

```Output
<timestamp>  INFO dsc_lib::dscresources::command_resource: 69: Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 850: Process 'pwsh' id <pid> exited with code 0
<timestamp> DEBUG dsc_lib::dscresources::command_resource: 72: Verifying output of get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp> ERROR dsc::resource_command: 67: JSON: expected value at line 1 column 1
```

We can demonstrate the failure independently of DSC. When you invoke the following snippet,
PowerShell hangs. A `[FileInfo]` object can be extremely large as the object contains references to
its parent folder, which references that object's parent folder, and so on.

```powershell
$fileInfo = Get-Item -Path 'Temp:/dsc/examples/PowerShellScript/output.txt'
$fileJson = ConvertTo-Json -Depth 9 -InputObject $fileInfo
# The following commands never run because the session hangs
$outputSize = [System.Text.Encoding]::UTF8.GetByteCount($fileJson) / 1MB
"The output JSON is {0} MB" -f [Math]::Round($outputSize, 2)
```

<!-- markdownlint-disable-next-line MD033 -->
You can cancel the command by pressing <kbd>Ctrl</kbd>+<kbd>C</kbd> in your console.

If you update the depth to `5` and invoke the command again, you can see that the size of the JSON
object is _substantial_.

```powershell
$fileInfo = Get-Item -Path 'Temp:/dsc/examples/PowerShellScript/output.txt'
$fileJson = ConvertTo-Json -Depth 5 -InputObject $fileInfo
# The following commands never run because the session hangs
$outputSize = [System.Text.Encoding]::UTF8.GetByteCount($fileJson) / 1MB
"The output JSON is {0} MB" -f [Math]::Round($outputSize, 2)
```

```Output
WARNING: Resulting JSON is truncated as serialization has exceeded the set depth of 5.
The output JSON is 58.37 MB
```

Instead of emitting complex objects directly, consider constructing your output objects
intentionally. For a comprehensive example of emitting structured output, see the
["Structure output for an idempotent instance"](#structure-output-for-an-idempotent-instance)
section of this article.

### Outputting arrays

By default, when a script emits an array as output, each item in the array is captured as a
separate item in the `output` property for the resource.

The following snippet shows the default behavior.

```powershell
$instance = @'
getScript: |-
   @('a', 'b', 'c')
   @(1, 2, 3)
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc resource get @arguments
```

```yaml
actualState:
  output:
  - a
  - b
  - c
  - 1
  - 2
  - 3
```

In the previous snippet, the script emitted two arrays:

1. An array containing three strings
1. An array containing three integers

The `output` for the resource included six separate items representing each of the items in the
emitted arrays in the order that the script emitted them.

The following snippet shows how you can use the [`Write-Object` cmdlet][03] with the
[`-NoEnumerate`][04] parameter to emit arrays from the script and keep them as arrays.

```powershell
$instance = @'
getScript: |-
  Write-Output -NoEnumerate @('a', 'b', 'c')
  Write-Output -NoEnumerate @(1, 2, 3)
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc resource get @arguments
```

```yaml
actualState:
  output:
  - - a
    - b
    - c
  - - 1
    - 2
    - 3
```

Now the output from the script shows two items in the `output` array. Each item is an array
containing three items.

## Discarding unwanted output

Every item emitted to the success stream is included in the `output` for the resource. To avoid
including unwanted data in the output you need to discard that data. To discard data from a
statement that would otherwise emit unwanted output, you can:

- Assign the statement to `$null`.
- Redirect the statement to `$null`.
- Cast the statement to `[void]`.
- Pipe the statement to `Out-Null`.

The first three options have nearly identical performance. Piping to `Out-Null` can be much slower
when looping over a large set of data.

The following snippet shows examples for discarding unwanted output in a script.

```powershell
$instance = @'
getScript: |-
  $filePath = 'Temp:/dsc/examples/PowerShellScript/output.txt'
  # Assign to `$null`
  $null = New-Item -Path $filePath -Force
  # Redirect to `$null`
  New-Item -Path $filePath -Force > $null
  # Cast to `[void]`
  [void](New-Item -Path $filePath -Force)
  # Pipe to `Out-Null`
  New-Item -Path $filePath -Force | Out-Null
  
  'this is the only output'  
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc resource get @arguments
```

```yaml
actualState:
  output:
  - this is the only output
```

## Output for `getScript`

For `getScript` you can emit any data to the PowerShell success stream that you want to surface to
the user. The emitted data is returned in the `actualState.output` field for the **Get** operation
result and the `beforeState.output` field for the **Set** operation result.

If you're defining the resource instance to idempotently manage the state of one or more system
components, ensure that the output you emit from `getScript` uses the same structure as the output
from `setScript` to make the results readable for the user.

Otherwise, return any data that you want to surface to the user. If you want to give the user more
information, you can [emit messages][05]. For comprehensive examples of emitting messages from your
script see [Invoke the PowerShellScript resource with trace messaging][06].

The following example shows how you can emit items from `getScript` to inform the user. For a
comprehensive example of structured output for an instance that idempotently manages system state,
see ["Structure output for an idempotent instance"](#structure-output-for-an-idempotent-instance)
in this article.

```powershell
$instance = @'
getScript: |-
  "Current context is interactive: {0}" -f [Environment]::UserInteractive
  "Current context is privileged: {0}" -f [Environment]::IsPrivilegedProcess
'@

dsc resource get --resource Microsoft.DSC.Transitional/PowerShellScript --input $instance
```

```yaml
actualState:
  output:
  - 'Current context is interactive: True'
  - 'Current context is privileged: False'
```

## Output for `testScript`

The `testScript` definition _must_ return a single boolean value - `$true` to indicate that the
system is in the desired state or `$false` otherwise.

Any of the following will cause the resource to raise an error when invoking the `testScript`:

- Not emitting any output at all to the success stream.
- Emitting any non-boolean data to the success stream.
- Emitting more than one boolean value to the success stream.

You can [emit messages][05] To indicate to the user how and why the
system isn't in the desired state. For detailed examples of emitting messages from your script
see [Invoke the PowerShellScript resource with trace messaging][06].

The following example shows how you can define `testScript` to check whether a file exists and
isn't empty. It emits info messages to clarify whether and how the instance is in the desired
state.

```powershell
$instance = @'
testScript: |-
  $filePath = 'Temp:/dsc/examples/PowerShellScript/output.txt'
  
  if (-not (Test-Path $filePath)) {
      Write-Verbose "The file '$filePath' doesn't exist"
      return $false
  }

  if ([string]::IsNullOrEmpty((Get-Content -Raw -Path $filePath))) {
      Write-Verbose "The file '$filePath' is empty"
      return $false
  }

  Write-Verbose "The file '$filePath' exists and contains content"
  $true
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/PowerShellScript'
    '--input', $instance
)

dsc --trace-level info resource test @arguments
```

```console
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Invoking test on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: The file 'Temp:/dsc/examples/PowerShellScript/output.txt' is empty
<timestamp>  INFO diff: key 'testScript' missing
desiredState:
  testScript: |-
    $filePath = 'Temp:/dsc/examples/PowerShellScript/output.txt'

    if (-not (Test-Path $filePath)) {
        Write-Verbose "The file '$filePath' doesn't exist"
        return $false
    }

    if ([string]::IsNullOrEmpty((Get-Content -Raw -Path $filePath))) {
        Write-Verbose "The file '$filePath' is empty"
        return $false
    }

    Write-Verbose "The file '$filePath' exists and contains content"
    $true
actualState:
  _inDesiredState: false
inDesiredState: false
differingProperties:
- testScript
```

For a more comprehensive example that idempotently manages system state see the
["Structure output for an idempotent instance"](#structure-output-for-an-idempotent-instance)
section of this article.

## Output for `setScript`

For `setScript` you can emit any data to the PowerShell success stream that you want to surface to
the user. The emitted data is returned in the `afterState.output` field for the **Set** operation
result.

If you're defining the resource instance to idempotently manage the state of one or more system
components, ensure that the output you emit from `setScript` uses the same structure as the output
from `getScript` to make the results readable for the user.

Otherwise, return any data that you want to surface to the user. If you want to give the user more
information, you can [emit messages][05]. For comprehensive examples of
emitting messages from your script see
[Invoke the PowerShellScript resource with trace messaging][06].

The following example shows how you can emit items from `setScript` to inform the user about how
the script is modifying the system.

```powershell
$instance = @'
setScript: |-
  $filePath = 'Temp:/dsc/examples/PowerShellScript/output.txt'
  $content  = 'Hello world'
  if (-not (Test-Path $filePath)) {
      "File '$filePath' doesn't exist - creating it"
      $null = New-Item -Path $filePath -Force
  }
  
  $currentContent = Get-Content -Raw -Path $filePath
  if ([string]::IsNullOrEmpty($currentContent)) {
      "File '$filePath' is empty - adding content"
      $content | Set-Content -Path $filePath -NoNewline
  } elseif ($currentContent -ne $content) {
      "File '$filePath' contains invalid content - overriding content"
      $content | Set-Content -Path $filePath -NoNewline
  } else {
    "File '$filePath' contains desired content'"
  }

  [ordered]@{
      initialContent = $currentContent
      finalContent   = $content
  }
'@

dsc resource set --resource Microsoft.DSC.Transitional/PowerShellScript --input $instance
```

```yaml
beforeState: {}
afterState:
  output:
  - File 'Temp:/dsc/examples/PowerShellScript/output.txt' is empty - adding conent
  - initialContent: null
    finalContent: Hello world
changedProperties:
- output
```

In this example, `beforeState` is an empty object because the instance doesn't define `getScript`.
The output from `setScript` includes two items. The first is a message indicating that the file
exists but is empty. The second item is an object showing both the initial content and final
content of the file.

For a comprehensive example of structured output for an instance that idempotently manages system
state, including defining `getScript` to populate the `beforeState` in the **Set** result, see the
[Structure output for an idempotent instance](#structure-output-for-an-idempotent-instance) section
of this article.

## Structure output for an idempotent instance

To return output that is readable for the user, consider returning only objects. Use property names
to orient the user when reviewing the output. Limit the depth of the object to no more than three
levels when possible.

The following example shows how you can return information about a JSON configuration file that
isn't managed by a specific DSC resource. It follows best practices by:

1. Implementing scripts for all three operations.
1. Returning a single structured object from `getScript`.
1. Returning a boolean for `testScript` and emitting trace messages to indicate _how_ the instance
   is out of the desired state.
1. Returning the same structured object from `setScript` as `getScript`.
1. Emitting trace messages to indicate which settings the `setScript` is modifying.

> [!NOTE]
> This example uses an ordered dictionary to represent the instance because the script properties
> are much longer and more detailed than earlier examples in this article. Defining the scripts
> this way makes it easier to review the script code than defining it all together in a YAML
> snippet.
>
> The `getScript` and `setScript` snippets define the output object as an ordered dictionary with
> the `[ordered]` type accelerator. This ensures that the emitted object always keeps the key-value
> pairs in the defined order. Defining the output object as a normal hashtable causes the ordering
> of the output object properties to be nondeterministic, which can make comparing results more
> difficult.
>
> You could also define the output object as a `[pscustomobject]` and use the `Add-Member` function
> to add more properties to the initial object.

First, define an ordered dictionary to represent the instance. Define the `input` for the scripts
the instance will use. In this example, the input data includes both the path to the file and the
settings to manage in that file.

```powershell
$instance = [ordered]@{
    input = [ordered]@{
        filePath = 'Temp:/dsc/examples/PowerShellScript/output.json'
        settings = [ordered]@{
            updateAutomatically = $true
            updateFrequency     = 30
        }
    }
}
```

Next, define `getScript` to retrieve the actual state of the configuration file. The script must
define a `param()` statement to accept the input data.

The script returns an object that always includes the `filePath` and `exists` properties.
`filePath` is identical to `input.filePath` for the instance. `exists` indicates whether the file
actually exists on the system.

If the file doesn't exist, that's all the information the instance can provide. The script returns
that data and stops processing.

If the file does exist, the output object also includes the `settings` and `lastWriteTime`
properties. `settings` is the contents of the file converted from JSON. `lastWriteTime` is the
actual last write time for the file itself.

```powershell
$instance.getScript = {
    param($inputData)
    
    $result = [ordered]@{
        filePath = $inputData.filePath
        exists   = Test-Path -Path $inputData.filePath
    }
    
    if (-not $result.exists) {
        Write-Verbose "Config file doesn't exist"
        return $result
    }
    Write-Verbose "Retrieving settings and last write time from config file"
    $fileInfo = Get-Item -Path $inputData.filePath
    $settings = Get-Content -Raw -Path $inputData.filePath | ConvertFrom-Json
    
    $result.settings      = $settings
    $result.lastWriteTime = $fileInfo.LastWriteTime
    
    $result
}.ToString()
```

The next snippet defines `testScript` for the instance. As with `getScript`, the script must define
a single parameter. Unlike `getScript`, this script must return exactly one boolean value.

The test script:

1. Checks whether the configuration file (`input.filePath`) exists. If it doesn't, the script emits
   an info message and returns `$false`.
1. Checks whether the configuration file is empty. If it is, the script emits an info message and
   returns `$false`.
1. Iterates over the key-value pairs for the desired settings (`input.settings`) to check whether
   each of them is in the desired state. If the desired setting isn't defined or is defined with
   an incorrect value the script emits an info message and marks the resource as noncompliant but
   _doesn't_ stop processing.

   This ensures that the instance can fully report on the desired settings instead of only reporting
   the first missing or incorrect setting.
1. Returns `$false` if any setting wasn't in the desired state and otherwise `$true`.

```powershell
$instance.testScript = {
    param($inputData)

    if (-not (Test-Path -Path $inputData.filePath)) {
        Write-Verbose "Config file doesn't exist"
        return $false
    }

    $content  = Get-Content -Raw -Path $inputData.filePath
    if ([string]::IsNullOrEmpty($content)) {
        Write-Verbose "Config file is empty"
        return $false
    }

    # Initialize variable for result. If any check fails, set to `$false`
    # From this point on we want to fully validate state for info messages to
    # the user instead of returning early.
    $inDesiredState = $true

    # Loop over the desired state to compare to actual settings
    $desiredSettings = $inputData.settings.psobject.Properties
    $actualSettings  = ($content | ConvertFrom-Json).psobject.Properties
    foreach ($setting in $desiredSettings) {
        $name          = $setting.Name
        $desiredValue  = $setting.Value
        $actualSetting = $actualSettings | Where-Object Name -EQ $name

        if ($null -eq $actualSetting) {
            Write-Verbose "Missing setting '$name'"
            $inDesiredState = $false
            continue
        }

        if ($actualSetting.Value -ne $setting.Value) {
            $message = "Expected setting '{0}' to be ``{1}`` but it is ``{2}``" -f @(
                $name
                $desiredValue
                $actualSetting.Value
            )
            Write-Verbose $message
            $inDesiredState = $false
        }
    }

    $inDesiredState
}.ToString()
```

To enforce the desired state, define the `setScript` for the instance. The script must define a
single parameter. To make the result for the **Set** operation readable the script emits the same
data structure as `getScript`.

The script is defined to be idempotent, only modifying the system if needed. It follows these steps:

1. Define the result object with `filePath` as the `input.filePath` value and `exists` as `true`.
1. Check whether the configuration file exists. If it doesn't, emit a message to indicate that the
   instance is creating the file. Then create the file and write the desired state settings
   (`input.settings`) into it. Populate the `settings` and `lastWriteTime` fields for the result
   object and then use the `return` keyword to emit the result and stop processing the script.
1. If the configuration file does exist retrieve the settings from it. Iterate over the desired
   state settings (`input.settings`). If the setting is missing or defined incorrectly, emit an
   info message and mark the instance as requiring an update with the `$shouldUpdate` variable.
   This ensures that the instance only modifies the file when the settings aren't in the desired
   state.

   > [!NOTE]
   > This is necessary for version `0.1.0` of this resource. In this release the resource doesn't
   > use the `testScript` to determine whether to actually invoke the `setScript`. The resource
   > _always_ invokes `setScript` when you invoke the **Set** operation for the resource or on a
   > configuration document containing an instance of the resource.

   If the setting is missing, add the desired state setting to the object representing the actual
   state. If the setting has the incorrect value, set that property on the same object to the
   desired state. This ensures that the resource doesn't inadvertently modify or remove any
   settings in the configuration file that the instance isn't managing (the setting is defined in
   the file but not `input.settings`).
1. If any of the desired state settings weren't defined in the configuration file or were defined
   with invalid values emit a message and update the file with the combined settings. Otherwise
   emit a message indicating that the configuration file didn't require any modification.
1. Update the result object to include the final settings and the last write time for the file and
   emit the result.

`setScript` returns the same structured output data as `getScript` regardless of whether the script
creates, updates, or doesn't modify the configuration file. This helps make the output for the
**Set** operation readable and enable directly comparing the `beforeState` and `afterState` fields
in the result.

```powershell
$instance.setScript = {
    param($inputData)

    $filePath = $inputData.filePath
    $settings = $inputData.settings
    $result   = [ordered]@{
        filePath      = $filePath
        exists        = $true
    }

    if (-not (Test-Path -Path $filePath)) {
        Write-Verbose "Creating config file with specified settings"
        $null = New-Item -Path $filePath -Force -Verbose
        $json = $settings | ConvertTo-Json
        $json | Out-File -FilePath $filePath -Encoding utf8NoBOM

        $result.settings      = $settings
        $result.lastWriteTime = Get-Item -Path $filePath |
            Select-Object -ExpandProperty LastWriteTime

        return $result
    }

    $content        = Get-Content -Raw -Path $filePath
    $actualSettings = $content | ConvertFrom-Json
    $shouldUpdate   = $false
    # Iterate over defined settings, updating the actual settings as needed.
    # Don't remove any non-managed settings, only enforce specified settings.
    # Set shouldUpdate to $true if any changes are needed, but wait to write
    # to the file until all changes are processed to avoid multiple writes.
    foreach ($setting in $settings.psobject.Properties) {
        $name   = $setting.Name
        $value  = $setting.Value
        Write-Verbose "Processing setting '$name' with desired value ``$value``"
        $actual = $actualSettings.psobject.Properties |
            Where-Object Name -EQ $name |
            Select-Object -First 1

        if ($null -eq $actual) {
            Write-Verbose "Adding setting '$name' as ``$value``"

            $shouldUpdate = $true
            $memberParams = @{
                InputObject = $actualSettings
                MemberType  = 'NoteProperty'
                Name        = $name
                Value       = $value
            }
            Add-Member @memberParams
        } elseif ($value -eq $actual.Value) {
            Write-Verbose "Setting '$name' is already set to ``$value``"
        } else {
            $message = "Changing setting '{0}' from ``{1}`` to ``{2}``" -f @(
                $name
                $actual.Value
                $value
            )
            Write-Verbose $message

            $shouldUpdate         = $true
            $actualSettings.$name = $value
        }
    }

    if ($shouldUpdate) {
        Write-Verbose "Updating config file with new settings"
        $json = $actualSettings | ConvertTo-Json
        $json | Out-File -FilePath $filePath -Encoding utf8NoBOM
    } else {
        Write-Verbose "Config file is already in the desired state. No update needed."
    }

    $result.settings      = $actualSettings
    $result.lastWriteTime = Get-Item -Path $filePath |
        Select-Object -ExpandProperty LastWriteTime

    $result
}.ToString()
```

With the instance fully defined, invoke the **Get** operation to ensure that returning the actual
state works as expected:

```powershell
dsc --trace-level info resource get --resource Microsoft.DSC.Transitional/PowerShellScript --input (
    ConvertTo-Json -InputObject $instance
)
```

```Output
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Invoking get 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Config file doesn't exist
actualState:
  output:
  - filePath: Temp:/dsc/examples/PowerShellScript/output.json
    exists: false
```

The output shows that the configuration file doesn't exist.

Next, invoke the **Set** operation to create the file:

```powershell
dsc --trace-level info resource set --resource Microsoft.DSC.Transitional/PowerShellScript --input (
    ConvertTo-Json -InputObject $instance
)
```

```Output
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Config file doesn't exist
<timestamp>  INFO PID <pid>: Creating config file with specified settings
<timestamp>  INFO PID <pid>: Performing the operation "Create File" on target "Destination: C:\Users\<username>\AppData\Local\Temp\dsc\examples\PowerShellScript\output.json".
<timestamp>  INFO diff: key 'updateAutomatically' is not an object
<timestamp>  INFO diff: key 'updateFrequency' is not an object
<timestamp>  INFO diff: key '_exist' is not an object
<timestamp>  INFO diff: key 'lastWriteTime' missing
<timestamp>  INFO diff: actual array missing expected item
<timestamp>  INFO diff: arrays differ for 'output'
beforeState:
  output:
  - filePath: Temp:/dsc/examples/PowerShellScript/output.json
    exists: false
afterState:
  output:
  - filePath: Temp:/dsc/examples/PowerShellScript/output.json
    exists: true
    settings:
      updateAutomatically: true
      updateFrequency: 30
    lastWriteTime: 2026-06-02T16:45:23.9746807-05:00
changedProperties:
- output
```

The emitted messages show that the configuration file doesn't exist and the resource is creating
it. The `beforeState` is populated by the `getScript` and shows that the file doesn't exist. The
`afterState` then shows that the instance created the file with the expected settings and includes
the last write time.

Invoking the **Set** operation again shows that the defined instance is idempotent:

```powershell
dsc --trace-level info resource set --resource Microsoft.DSC.Transitional/PowerShellScript --input (
    ConvertTo-Json -InputObject $instance
)
```

```Output
<timstamp>  INFO Trace-level is Info
<timstamp>  INFO Discovering 'Extension' using filter: *
<timstamp>  INFO Discovering 'Resource' using filter: *
<timstamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timstamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timstamp>  INFO PID <pid>: Retrieving settings and last write time from config file
<timstamp>  INFO PID <pid>: Processing setting 'updateAutomatically' with desired value `True`
<timstamp>  INFO PID <pid>: Setting 'updateAutomatically' is already set to `True`
<timstamp>  INFO PID <pid>: Processing setting 'updateFrequency' with desired value `30`
<timstamp>  INFO PID <pid>: Setting 'updateFrequency' is already set to `30`
<timstamp>  INFO PID <pid>: Config file is already in the desired state. No update needed.
beforeState:
  output:
  - filePath: Temp:/dsc/examples/PowerShellScript/output.json
    exists: true
    settings:
      updateAutomatically: true
      updateFrequency: 30
    lastWriteTime: 2026-06-02T16:45:23.9746807-05:00
afterState:
  output:
  - filePath: Temp:/dsc/examples/PowerShellScript/output.json
    exists: true
    settings:
      updateAutomatically: true
      updateFrequency: 30
    lastWriteTime: 2026-06-02T16:45:23.9746807-05:00
changedProperties: []
```

The output in `beforeState` and `afterState` is identical and `changedProperties` is an empty
array. The emitted messages clarify that the instance checked each setting in the configuration
file and found them compliant to the desired state.

Finally, update `input.settings` by:

- Removing `updateAutomatically`
- Updating `updateFrequency` to `45`
- Adding `logLevel` as `info`

Then invoke the resource again to see how the instance updates the configuration file.

```powershell
$instance.input.settings.Remove('updateAutomatically')
$instance.input.settings.updateFrequency = 45
$instance.input.settings.logLevel        = 'info'

dsc --trace-level info resource set --resource Microsoft.DSC.Transitional/PowerShellScript --input (
    ConvertTo-Json -InputObject $instance
)
```

```Output
<timestamp>  INFO Trace-level is Info
<timestamp>  INFO Discovering 'Extension' using filter: *
<timestamp>  INFO Discovering 'Resource' using filter: *
<timestamp>  INFO No results returned for discovery extension 'Microsoft.PowerShell/Discover'
<timestamp>  INFO Getting current state for set by invoking get on 'Microsoft.DSC.Transitional/PowerShellScript' using 'pwsh'
<timestamp>  INFO PID <pid>: Retrieving settings and last write time from config file
<timestamp>  INFO PID <pid>: Processing setting 'updateFrequency' with desired value `45`
<timestamp>  INFO PID <pid>: Changing setting 'updateFrequency' from `30` to `45`
<timestamp>  INFO PID <pid>: Processing setting 'logLevel' with desired value `info`
<timestamp>  INFO PID <pid>: Adding setting 'logLevel' as `info`
<timestamp>  INFO PID <pid>: Updating config file with new settings
<timestamp>  INFO diff: key 'logLevel' missing
<timestamp>  INFO diff: actual array missing expected item
<timestamp>  INFO diff: arrays differ for 'output'
beforeState:
  output:
  - filePath: Temp:/dsc/examples/PowerShellScript/output.json
    exists: true
    settings:
      updateAutomatically: true
      updateFrequency: 30
    lastWriteTime: 2026-06-02T16:45:23.9746807-05:00
afterState:
  output:
  - filePath: Temp:/dsc/examples/PowerShellScript/output.json
    exists: true
    settings:
      updateAutomatically: true
      updateFrequency: 45
      logLevel: info
    lastWriteTime: 2026-06-02T16:53:39.2138244-05:00
changedProperties:
- output
```

The emitted messages indicate that the instance only checked the `updateFrequency` and `logLevel`
settings - it didn't enforce `updateAutomatically`. The messages show that the instance updated
`updateFrequency` from `30` to `45` and added the missing `logLevel` setting.

The result object again shows how `beforeState` differs from `afterState`, confirming that the
instance did modify system state.

<!-- Link reference definitions -->
[01]: ../index.md
[02]: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_output_streams?view=powershell-7.6#success-stream
[03]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-output
[04]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/write-output#-noenumerate
[05]: ../index.md#emitting-messages
[06]: ./invoke-with-messaging.md
