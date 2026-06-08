---
description: Microsoft.DSC.Transitional/WindowsPowerShellScript resource reference documentation
ms.date:     07/07/2025
ms.topic:    reference
title:       Microsoft.DSC.Transitional/WindowsPowerShellScript
---

<!-- markdownlint-disable MD025 MD033 -->

# Microsoft.DSC.Transitional/WindowsPowerShellScript

## Synopsis

Enable running Windows PowerShell 5.1 scripts inline.

> [!IMPORTANT]
> The `Microsoft.DSC.Transitional/WindowsPowerShellScript` resource is intended as a temporary
> transitional resource while defining DSC resources for your needs.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Transitional, Windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.DSC.Transitional/WindowsPowerShellScript
    properties:
      # Optional properties
      getScript: string
      setScript: string
      testScript: string
      input: anyOf # string, boolean, integer, object, array, null
      output: array
      _inDesiredState: boolean # or null
```

## Description

The `Microsoft.DSC.Transitional/WindowsPowerShellScript` resource enables you to run Windows PowerShell 5.1 scripts inline
as part of your DSC configuration. This resource is useful for executing PowerShell logic that hasn't
been fully transitioned to a dedicated DSC resource.

The resource allows you to:

- Define separate PowerShell scripts for **Get**, **Set**, and **Test** operations.
- Pass input data to the scripts.
- Receive output data from the scripts.
- Control the desired state behavior through the `_inDesiredState` property.

The properties you define determine how the resource behaves.

- If you don't define `getScript`, the `actualState` field in **Get** operation results and
  `beforeState` field in **Set** operation results is always an empty object (`{}`).
- If you don't define `testScript`, the `inDesiredState` field for **Test** operation results is
  always `true`.
- If you don't define `setScript`, the `afterState` field in **Set** operation results is always
  an empty object (`{}`).
- If you define `input`, every script property you define _must_ start with a `param()` statement
  that defines a single parameter. The value for `input` is _always_ passed to the scripts when the
  resource invokes them.
  
  When the instance _doesn't_ define `input` the script properties must **not** include a `param()`
  statement.

> [!NOTE]
> This resource always invokes the script properties in PowerShell (`pwsh`). To define a resource
> instance with script properties that execute in Windows PowerShell (`powershell.exe`), see
> [`Microsoft.DSC.Transitional/PowerShellScript`][01].

### Defining script properties

For an instance to be functional you must define one or more script properties:

- Define `getScript` to retrieve actual system state with the **Get** operation or to show how the
  instance modified the system during a **Set** operation.
- Define `testScript` to indicate whether the system is in the desired state with the **Test**
  operation.

  > [!IMPORTANT]
  > Version `0.1.0` of the resource does _not_ invoke the `testScript` to determine whether to
  > invoke the `setScript`. The resource always invokes `setScript` for the **Set** operation.
  >
  > Ensure that you define the `setScript` to be idempotent or include a check before making any
  > changes to the system to avoid unnecessary processing and unintended behaviors.

- Define `setScript` to modify the system with the **Set** operation. You can use this resource to
  define an instance that performs a specific task, such as warming a cache or clearing logs, or to
  enforce a specific desired state for any number of system components.

  In either case, consider [emitting messages](#emitting-messages) to the user that helps them
  understand what the instance is doing during an operation.

  If you are using the resource instance to enforce a specific desired state you should:

  1. Emit one or more output objects representing the final state of the system components the
     instance is modifying.
  1. Define `getScript` to emit the same data structures as output objects representing the actual
     state of the system components the instance is managing.

  This ensures that the user can more easily compare the `beforeState` and `afterState` fields of
  the **Set** operation result to see how the instance modified the system.

The following subsections provide more information on input, error handling, output, and emitting
messages from within the script properties.

#### Handling input

To pass input to a script, you must:

1. Define the script property with a `param()` statement that specifies a single parameter.
   Omitting the `param()` statement, defining an empty `param()` statement, or defining more than
   one parameter all cause the resource to fail.
1. Define the [`input`](#input) property for the resource instance with a non-null value. When you
   omit the `input` property or define it with a null value, like `input: null`, the resource
   raises an error causing the operation to fail.

The data bound to the script parameter is the result of using the `ConvertFrom-Json` cmdlet on the
value for the `input` property of the resource instance.

You can define the script parameter with a type, like `[string[]]` when the script expects the input
as an array of strings. PowerShell's normal parameter binding and type conversion behavior applies
to the script parameter. If the input data can't be converted to the defined type then the script
fails and raises an error indicating that the input data was invalid.

You can also apply [validation attributes][02] to the parameter to further validate that the input
data is correct for your script.

For detailed examples of using input data with this resource, see
[Invoke the WindowsPowerShellScript resource with input data][03].

#### Handling errors

This resource invokes the PowerShell scripts with the [`$ErrorActionPreference` variable][04] set
to `Stop`. By default, _any_ error raised by the script, regardless of whether it's terminating,
stops script execution.

You can control whether script execution continues on an error message in two ways:

1. Specify the [`-ErrorAction` common parameter][05] for any command you expect to fail. Specify
   the value for the parameter as `Continue` to emit the error message or `Ignore` to skip the
   error message. In either case, execution will continue after the error.
1. Use a [`try`/`catch` statement][06] to add error handling for errors. When a statement in the
   `try` block raises an error, the code in the `catch` block will execute before the code in the
   `finally` block (if defined). Unless code in the `catch` or `finally` blocks raises an error,
   the script will continue to execute.

Providing error handling enables you to emit better information for users when something goes wrong
with the script behavior.

However, even when you provide handling for errors, like using a `try`/`catch` statement or passing
`-ErrorAction Ignore` to a command you expect to fail, the resource considers the operation to have
failed. The resource doesn't populate the `output` property for failed scripts.

There is no way with the current version of the resource for a script to raise any errors and _not_
fail. You can only provide better diagnostics for the user in the event of a failure.

For detailed examples of emitting errors from scripts, see ["Emitting errors"][07] in
[Invoke the WindowsPowerShellScript resource with trace messaging][08].

#### Returning output

Any objects emitted by the script for an operation are converted to JSON with the `ConvertTo-Json`
cmdlet and appended to the `output` property array returned by the resource. The ordering of the
items in `output` is the same that they were emitted by the script.

You can emit any number of items. You don't need to use any specific PowerShell cmdlet to emit
output for this resource. Any output from a PowerShell statement that isn't redirected or captured
as a variable is automatically included in the output.

You can prevent statements from emitting output by assigning them to `$null`. For example, if your
script uses the `New-Item` cmdlet to create a file, the output for that command is emitted from
your script by default. To avoid emitting that data, you could use the following snippet:

```powershell
$null = New-Item -Path $filePath
```

To provide more readable results to users, consider only emitting a single structured object from
both `getScript` and `setScript`. Emitting an object with descriptive property names makes it
easier to compare the `beforeState` and `afterState` fields for a **Set** operation result. Using
the same data structure also enables DSC to correctly determine the `changedProperties` field for
the **Set** operation result. If the output from `getScript` and `setScript` are identical then
`changedProperties` is an empty array.

For `testScript`, be sure to _only_ and _always_ emit a single boolean value (`$true` or `$false`).
If `testScript` emits any non-boolean value, more than one boolean value, or no values at all then
the resource considers the operation to have failed and raises an error.

For comprehensive examples showing how to emit and control output from scripts, see
[Invoke the WindowsPowerShellScript resource with output data][09].

#### Emitting messages

The following table maps DSC's tracing levels to PowerShell output streams and `Write-*` cmdlets:

| DSC trace level | PowerShell stream |      PowerShell cmdlets       |
|:---------------:|:-----------------:|:-----------------------------:|
|        -        |      Success      |        `Write-Output`         |
|     `error`     |       Error       |         `Write-Error`         |
|     `warn`      |      Warning      |        `Write-Warning`        |
|     `info`      |      Verbose      | `Write-Verbose`, `Write-Host` |
|     `debug`     |       Debug       |         `Write-Debug`         |
|     `trace`     |    Information    |      `Write-Information`      |

> [!IMPORTANT]
> Remember that _any_ error emitted from the script causes the resource and DSC to consider the
> script execution to have failed, even when the script continued after an error.

For comprehensive examples of emitting messages from scripts, see
[Invoke the WindowsPowerShellScript resource with trace messaging][08].

## Requirements

- The resource is only usable on a Windows system.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `test` - You can use the resource to test whether an instance is in the desired state.

For more information about resource capabilities, see [DSC resource capabilities][00].

## Examples

1. [Configure a system with the WindowsPowerShellScript resource][10] - Shows how to use this
   resource in a configuration document.
1. [Invoke the WindowsPowerShellScript resource with input data][11] - Shows how to pass data to
   this resource.
1. [Invoke the WindowsPowerShellScript resource with output data][09] - Shows how to return data
   from this resource.
1. [Invoke the WindowsPowerShellScript resource with trace messaging][08] - Shows how to emit DSC
   trace messages from this resource.

## Properties

The following list describes the properties for the resource.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [getScript](#getscript) - The Windows PowerShell script to run during the **Get** operation.
  - [setScript](#setscript) - The Windows PowerShell script to run during the **Set** operation.
  - [testScript](#testscript) - The Windows PowerShell script to run during the **Test** operation.
  - [input](#input) - Input data to pass to the Windows PowerShell scripts.
  - [output](#output) - Output data returned from the Windows PowerShell scripts.
  - [_inDesiredState](#_indesiredstate) - Indicates whether the resource instance is in the desired state.

### getScript

<details><summary>Expand for <code>getScript</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the Windows PowerShell script to execute during the **Get** operation. This property is
never returned by the resource. The resource invokes the script this property defines for the
**Get** operation and to populate the `beforeState` for a **Set** operation.

This script should return the current state of the instance. The script can access input data and
should return relevant state information. _Every_ item the script emits to the PowerShell success
stream is inserted into the [`output`](#output) property.

When possible, prefer emitting a single structured object to the success stream. This makes reading
the `actualState` for a **Get** operation result and the `beforeState` for a **Set** operation
result easier for users.

### setScript

<details><summary>Expand for <code>setScript</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the Windows PowerShell script to execute during the **Set** operation. This script should
configure the system to match the desired state. The script can access input data and should
perform the necessary changes to bring the system into compliance.

If the instance defines the [`getScript`](#getscript) property to return data then this property
_should_ return data in the same order and structure. The result object for the **Set** operation
includes `beforeState` (populated by the output for `getScript`) and `afterState` (populated by the
output for `setScript`). Keeping the output order and structure the same for both scripts enables
easier comparison of the changes in resource state.

### testScript

<details><summary>Expand for <code>testScript</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the Windows PowerShell script to execute during the **Test** operation. This script should
determine whether the system is in the desired state and return appropriate state information. The
script can access input data and should return a single boolean value of `$true` or `$false`.

The script should _not_ emit any other data for output. Emitting more data than a single boolean
value or emitting a non-boolean value causes the resource to raise an error.

Instead, [emit messages](#emitting-messages) to indicate how and why the instance is out of the
desired state.

> [!IMPORTANT]
> In version `0.1.0` for the resource, this script is _only_ invoked for the **Test** operation
> when you use the `dsc config test` or `dsc resource test` commands. When you invoke the **Set**
> operation the resource _always_ invokes the [`setScript`](#setscript) even when `testScript`
> would report that the resource is in the desired state.

### input

<details><summary>Expand for <code>input</code> property metadata</summary>

```yaml
Type             : anyOf (string, boolean, integer, object, array, null)
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines input data to pass to the PowerShell scripts. This can be any of the following JSON data
types:

- `string`
- `boolean`
- `integer`
- `object`
- `array`

The input data is available to every script property and can be used to parameterize script
behavior.

When passing input data to a script, always define the `params` keyword with a single named
parameter, like `params($inputData)`. The resource binds the value from the `input` property to
that parameter.

The value for this property affects how it is passed to the PowerShell scripts for the resource:

| JSON value type | Bound PowerShell parameter value |
|:---------------:|:--------------------------------:|
|    `string`     |            `[String]`            |
|    `object`     |        `[PSCustomObject]`        |
|     `array`     |           `[Object[]]`           |
|    `integer`    |            `[Int64]`             |
|    `number`     |             Invalid †            |
|    `boolean`    |           `[Boolean]`            |
|     `null`      |             Invalid †            |

> [!NOTE]
> Passing a number with a fractional part, such as `1.23`, or `null` is invalid for the top-level
> value of the `input` field. However, you can pass numbers and `null` values nested as object
> properties or array items.
>
> For example, `input: 1.23` is invalid while `input: {"num": 1.23}` and `input: [1.23]` are valid.
> Similarly, `input: null` is invalid while `input: {nested: null}` and `input: [null]` are both
> valid.

If you define your scriptblock parameters without providing a type for the input data, like
`params($inputData)`, the type for that parameter is exactly as described in the prior table. You
can also define a type for the parameter, which causes PowerShell to cast the input data to the
given type. For example, `params([string[]]$inputData)` will cast the value for `input` to an array
of strings.

For comprehensive examples of how to use input data with this resource, see
[Invoke the WindowsPowerShellScript resource with input data][03].

### output

<details><summary>Expand for <code>output</code> property metadata</summary>

```yaml
Type             : array
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines output data returned from the Windows PowerShell scripts. This property contains the
results of script execution and can include any data that the scripts choose to return.

Every object emitted to the PowerShell success output stream is inserted into the `output` for the
operation in the order that the scriptblock emits those objects. The emitted items are
automatically converted to JSON values by the resource. Don't use the `ConvertTo-Json` cmdlet to
transform the items yourself.

When emitting objects with nested properties the resource will emit the object up 9 levels deep.
Objects with more deep nesting fail to serialize correctly into JSON.

Where possible, limit the output data to the value you need. You can use the `Select-Object` cmdlet
to select only the required properties or create a custom object to represent the output data.

> [!IMPORTANT]
> This resource doesn't populate the `output` property for failed scripts. The resource considers
> a script to have failed when it emits _any_ errors, even when those errors are explicitly handled.
> For more information, see the [Handling errors](#handling-errors) section of this documentation.

Using the `Write-*` cmdlets to emit messages to PowerShell's other output streams doesn't populate
the `output` property. Instead, those messages are surfaced through DSC's tracing. For more
information, see the [Emitting messages](#emitting-messages) section of this documentation.

For comprehensive examples of how to return output data with this resource, see
[Invoke the WindowsPowerShellScript resource with output data][09].

### _inDesiredState

<details><summary>Expand for <code>_inDesiredState</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
DefaultValue     : null
```

</details>

Indicates whether the resource is in the desired state. This property is only returned when a
caller invokes the **Test** operation for the resource. The value of this property depends on
whether the resource defines the [`testScript](#testscript) property:

1. When the resource instance defines `testScript`, DSC invokes that script and uses the boolean
   result it returns as the value of this property.
1. When the resource instance doesn't define `testScript`, the value is `true`.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "type": "object",
  "properties": {
    "getScript": {
      "type": ["string", "null"]
    },
    "setScript": {
      "type": ["string", "null"]
    },
    "testScript": {
      "type": ["string", "null"]
    },
    "input": {
      "type": ["string", "boolean", "integer", "object", "array", "null"]
    },
    "output": {
      "type": ["array", "null"]
    },
    "_inDesiredState": {
      "type": ["boolean", "null"],
      "default": null
    }
  },
  "additionalProperties": false
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - PowerShell script execution failed
- [2](#exit-code-2) - PowerShell exception occurred
- [3](#exit-code-3) - Script had errors

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the Windows PowerShell script execution failed. When the resource returns this exit code,
it also emits an error message with details about the execution failure.

### Exit code 2

Indicates a Windows PowerShell exception occurred during script execution. When the resource
returns this exit code, it writes the error to the console.

### Exit code 3

Indicates the script had errors, typically due to missing or invalid input data. This exit code is
commonly returned when required input parameters are not provided to the PowerShell scripts or when
the input data is in an unexpected format.

## See also

- [Microsoft.DSC.Transitional/RunCommandOnSet][12]
- [Microsoft.DSC.Transitional/PowerShellScript][13]

<!-- Link definitions -->
[00]: ../../../../concepts/dsc/resource-capabilities.md
[01]: ../WindowsPowerShellScript/index.md
[02]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_functions_advanced_parameters#parameter-and-variable-validation-attributes
[03]: ./examples/invoke-with-input-data.md
[04]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_preference_variables#erroractionpreference
[05]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_commonparameters#-erroraction
[06]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_try_catch_finally
[07]: ./examples/invoke-with-messaging.md#emitting-errors
[08]: ./examples/invoke-with-messaging.md
[09]: ./examples/invoke-with-output-data.md
[10]: ./examples/configure-with-script.md
[11]: ./examples/powershell-script-with-input-output.md
[12]: ../RunCommandOnSet/index.md
[13]: ../PowerShellScript//index.md
