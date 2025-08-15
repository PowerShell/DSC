---
description: Microsoft.DSC.Transitional/PowerShellScript resource reference documentation
ms.date:     07/07/2025
ms.topic:    reference
title:       Microsoft.DSC.Transitional/PowerShellScript
---

# Microsoft.DSC.Transitional/PowerShellScript

## Synopsis

Enable running PowerShell 7 scripts inline.

> [!IMPORTANT]
> The `psscript` command and `Microsoft.DSC.Transitional/PowerShellScript` resource
> is intended as a temporary transitional resource while migrating DSCv3 resources for
> your needs.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Transitional, Windows, Linux, MacOS]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.DSC.Transitional/PowerShellScript
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

The `Microsoft.DSC.Transitional/PowerShellScript` resource enables you to run PowerShell 7 scripts inline
as part of your DSC configuration. This resource is useful for executing PowerShell logic that hasn't
been fully transitioned to a dedicated DSC resource.

The resource allows you to:

- Define separate PowerShell scripts for get, set, and test operations
- Pass input data to the scripts
- Receive output data from the scripts
- Control the desired state behavior through the `_inDesiredState` property

> [!NOTE]
> This resource requires PowerShell 7 (`pwsh`) to be installed on the system.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `test` - You can use the resource to test whether an instance is in the desired state.

This resource implements its own test functionality through the `testScript` property.
For more information about resource capabilities, see [DSC resource capabilities][00].

## Examples

1. [Run a simple PowerShell script][01] - Shows how to run a basic PowerShell script.
2. [PowerShell script with input and output][02] - Shows how to pass data to and from PowerShell scripts.

## Properties

The following list describes the properties for the resource.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [getScript](#getscript) - The PowerShell script to run during the get operation.
  - [setScript](#setscript) - The PowerShell script to run during the set operation.
  - [testScript](#testscript) - The PowerShell script to run during the test operation.
  - [input](#input) - Input data to pass to the PowerShell scripts.
  - [output](#output) - Output data returned from the PowerShell scripts.
  - [_inDesiredState](#_indesiredstate) - Indicates whether the resource is in the desired state.

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

Defines the PowerShell script to execute during the **Get** operation. This script should return
the current state of the resource. The script can access input data and should return relevant
state information.

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

Defines the PowerShell script to execute during the **Set** operation. This script should
configure the system to match the desired state. The script can access input data and
should perform the necessary changes to bring the system into compliance.

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

Defines the PowerShell script to execute during the **Test** operation. This script should
determine whether the system is in the desired state and return appropriate state information.
The script can access input data and should return state information including the `_inDesiredState` property.

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

Defines input data to pass to the PowerShell scripts. This can be any valid JSON data type
including strings, booleans, integers, objects, arrays, or null. The input data is available
to all scripts (get, set, test) and can be used to parameterize script behavior.

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

Defines output data returned from the PowerShell scripts. This property contains the results
of script execution and can include any data that the scripts choose to return.

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

Indicates whether the resource is in the desired state. This property is typically set by
the `testScript` and used by DSC to determine whether the `setScript` needs to be executed.
When `null` (default), DSC will rely on the test script logic to determine state.

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

Indicates the PowerShell script execution failed. When the resource returns this
exit code, it also emits an error message with details about the execution failure.

### Exit code 2

Indicates a PowerShell exception occurred during script execution. When the resource returns this
exit code, it writes the error to the console.

### Exit code 3

Indicates the script had errors, typically due to missing or invalid input data.
This exit code is commonly returned when required input parameters are not provided
to the PowerShell scripts or when the input data is in an unexpected format.

## See also

- [Microsoft.DSC.Transitional/RunCommandOnSet][03]
- [Microsoft.DSC.PowerShell][04]
- [Microsoft.Windows.WindowsPowerShell][05]

<!-- Link definitions -->
[00]: ../../../../concepts/dsc/resource-capabilities.md
[01]: ./examples/run-simple-powershell-script.md
[02]: ./examples/powershell-script-with-input-output.md
[03]: ../RunCommandOnSet/index.md
[04]: ../../PowerShell/index.md
[05]: ../../../../Microsoft/Windows/WindowsPowerShell/index.md
