---
description: >
  Example showing how to pass input data to a WindowsPowerShellScript resource
  and access properties, array elements, and nested values inside the script.
ms.date:     05/10/2026
ms.topic:    reference
title:       Invoke the WindowsPowerShellScript resource with input data
---

<!-- markdownlint-disable MD025 -->

# Invoke the WindowsPowerShellScript resource with input data

These examples show how you can pass input data to the
[`Microsoft.DSC.Transitional/WindowsPowerShellScript` resource][01] and how to bind that data to your
script with a [`param()` statement][02].

## Input data types

The following examples show how data input is bound to the parameters for a defined scriptblock
when the parameter isn't defined with a specific type.

The data that the resource passes to a script is first converted from the JSON input that DSC sends
with the [`ConvertFrom-Json` cmdlet][03].

### Passing string input data

When you define `input` as a string value, the parameter for the script is a `[string]` object.

```powershell
$instance = @'
input: hello world
getScript: |-
  param($inputData)

  [ordered]@{
      boundDataType =  "[$($inputData.GetType().FullName)]"
      boundDataValue = $inputData
  }
'@

$arguments = @(
   '--resource', 'Microsoft.DSC.Transitional/WindowsPowerShellScript'
   '--input', $instance
)
dsc resource get @arguments
```

```yaml
actualState:
  output:
  - boundDataType: '[System.String]'
    boundDataValue: hello world
```

### Passing integer input data

When you define `input` as an integer value, the parameter for the script is an `[Int64]` value.

```powershell
$instance = @'
input: 10
getScript: |-
  param($inputData)

  [ordered]@{
      boundDataType =  "[$($inputData.GetType().FullName)]"
      boundDataValue = $inputData
  }
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/WindowsPowerShellScript'
    '--input', $instance
)
dsc resource get @arguments
```

```yaml
actualState:
  output:
  - boundDataType: '[System.Int64]'
    boundDataValue: 10
```

### Passing boolean input data

When you define `input` as a boolean value, the parameter for the script is a `[Boolean]` value.

```powershell
$instance = @'
input: true
getScript: |-
  param($inputData)

  [ordered]@{
      boundDataType =  "[$($inputData.GetType().FullName)]"
      boundDataValue = $inputData
  }
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/WindowsPowerShellScript'
    '--input', $instance
)
dsc resource get @arguments
```

```yaml
actualState:
  output:
  - boundDataType: '[System.Boolean]'
    boundDataValue: true
```

### Passing array input data

When you define `input` as a string value, the parameter for the script is a `[Object[]]` array.
The items in the array are data types as emitted by the [`ConvertFrom-Json` cmdlet][03].

```powershell
$instance = @'
input:
- hello world
- 10
- 1.23
- true
- null
- nested: object
- - nested
  - array
getScript: |-
  param($inputData)

  $inputData | ForEach-Object -Begin { $i = 0 } -Process {
      [ordered]@{
          boundDataItemIndex = $i
          boundDataItemType  = if ($null -eq $_) {
                                   '$null'
                               } else {
                                   "[$($_.GetType().FullName)]"
                               }
          boundDataItemValue = $_
      }
      $i++
  }
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/WindowsPowerShellScript'
    '--input', $instance
)
dsc resource get @arguments
```

```yaml
actualState:
  output:
  - boundDataItemIndex: 0
    boundDataItemType: '[System.String]'
    boundDataItemValue: hello world
  - boundDataItemIndex: 1
    boundDataItemType: '[System.Int64]'
    boundDataItemValue: 10
  - boundDataItemIndex: 2
    boundDataItemType: '[System.Double]'
    boundDataItemValue: 1.23
  - boundDataItemIndex: 3
    boundDataItemType: '[System.Boolean]'
    boundDataItemValue: true
  - boundDataItemIndex: 4
    boundDataItemType: $null
    boundDataItemValue: null
  - boundDataItemIndex: 5
    boundDataItemType: '[System.Management.Automation.PSCustomObject]'
    boundDataItemValue:
      nested: object
  - boundDataItemIndex: 6
    boundDataItemType: '[System.Object[]]'
    boundDataItemValue:
    - nested
    - array
```

### Passing object input data

When you define `input` as an object value, the parameter for the script is a `[pscustomobject]`.
The values for each property of the object are data types as emitted by the
[`ConvertFrom-Json` cmdlet][03].

```powershell
$instance = @'
input:
  string: hello world
  integer: 10
  number: 1.23
  boolean: true
  "null": null
  nestedObject:
    foo: bar
  nestedArray:
  - nested
  - array
getScript: |-
  param($inputData)

  $inputData.psobject.Properties | ForEach-Object -Process {
      [ordered]@{
          boundDataPropertyName  = $_.Name
          boundDataPropertyType  = "[$($_.TypeNameOfValue)]"
          boundDataPropertyValue = $_.Value
      }
  }
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/WindowsPowerShellScript'
    '--input', $instance
)
dsc resource get @arguments
```

```yaml
actualState:
  output:
  - boundDataPropertyName: string
    boundDataPropertyType: '[System.String]'
    boundDataPropertyValue: hello world
  - boundDataPropertyName: integer
    boundDataPropertyType: '[System.Int64]'
    boundDataPropertyValue: 10
  - boundDataPropertyName: number
    boundDataPropertyType: '[System.Double]'
    boundDataPropertyValue: 1.23
  - boundDataPropertyName: boolean
    boundDataPropertyType: '[System.Boolean]'
    boundDataPropertyValue: true
  - boundDataPropertyName: 'null'
    boundDataPropertyType: '[System.Object]'
    boundDataPropertyValue: null
  - boundDataPropertyName: nestedObject
    boundDataPropertyType: '[System.Management.Automation.PSCustomObject]'
    boundDataPropertyValue:
      foo: bar
  - boundDataPropertyName: nestedArray
    boundDataPropertyType: '[System.Object[]]'
    boundDataPropertyValue:
    - nested
    - array
```

## Casting input data

When you define the parameters for a scriptblock, you can specify a type for the input data. The
script uses PowerShell's [parameter type conversion][04] to try to convert the input
data. If the type conversion is impossible for the input data, PowerShell raises an error and the
operation fails.

The following example shows how you can convert the input data to a given type. In this case, it
converts every item in the input data into a `[datetime]` object.

```powershell
$instance = @'
input:
  - 2026-01-02
  - 01/20/2026
getScript: |-
  param([datetime[]]$inputData)

  $inputData | ForEach-Object {
      [ordered]@{
        InputDate = $_
        NextDate  = $_.AddDays(1)
      }
  }
'@

$arguments = @(
    '--resource', 'Microsoft.DSC.Transitional/WindowsPowerShellScript'
    '--input', $instance
)
dsc resource get @arguments
```

```yaml
actualState:
  output:
  - InputDate: 2026-01-02T00:00:00
    NextDate: 2026-01-03T00:00:00
  - InputDate: 2026-01-20T00:00:00
    NextDate: 2026-01-21T00:00:00
```

## Input related errors

Passing input to a script has several requirements:

1. The script property for the resource must use the `param()` statement to define exactly one
   parameter.
1. The `input` property for the resource must be defined with a non-null value.
1. If the `param()` statement defines a type for the input data, the value for the `input` property
   of the instance must be convertible to that type.

The resource raises an error and prevents the script from executing when any of these requirements
aren't met by the resource instance definition.

### Error: input provided but script has no parameters

If you provide a value for `input` but the script does not define a `param()` statement, the
resource exits with code `2` and emits the following error message:

```plaintext
Input was provided but script does not have a parameter to accept input.
```

```powershell
$instance = @'
getScript: |
  "Script without parameters"
input: oops
'@

dsc resource get --resource Microsoft.DSC.Transitional/WindowsPowerShellScript --input $instance
```

```Output
<timestamp> ERROR PID <pid>: Input was provided but script does not have a parameter to accept input.
<timestamp> ERROR Failed to run process 'powershell': Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
<timestamp> ERROR Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
```

### Error: Script defines a parameter but no input provided

If the script defines a `param()` statement but no `input` is specified for the instance, the
resource exits with code `2` and emits the following error message:

```plaintext
Script has a parameter '<parameter-name>' but no input was provided.
```

```powershell
$instance = @'
getScript: |
  param($inputObj)
  "This will not run"
'@

dsc resource get --resource Microsoft.DSC.Transitional/WindowsPowerShellScript --input $instance
```

```Output
<timestamp> ERROR PID <pid>: Script has a parameter 'inputObj' but no input was provided.
<timestamp> ERROR Failed to run process 'powershell': Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
<timestamp> ERROR Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
```

### Error: Script defines more than one parameter

If the script defines a `param()` statement with two or more parameters, the resource exits with
code `1` and emits the following error message:

```plaintext
Script must have exactly one parameter.
```

```powershell
$instance = @'
input:
- first
- second
getScript: |-
  param($a, $b)

  [ordered]@{
      a = $a
      b = $b
  }
'@

dsc resource get --resource Microsoft.DSC.Transitional/WindowsPowerShellScript --input $instance
```

```Output
<timestamp> ERROR PID 23764: Script must have exactly one parameter.
<timestamp> ERROR Failed to run process 'powershell': Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
<timestamp> ERROR Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
```

### Error: Script defines a typed parameter but input is invalid

If the script defines the `param()` statement with a parameter that has a defined type that the
input data can't convert into, the resource raises an error message about an argument
transformation failure.

```powershell
$instance = @'
input: foo
getScript: |-
  param([int]$inputData)

  $inputData
'@

dsc resource get --resource Microsoft.DSC.Transitional/WindowsPowerShellScript --input $instance
```

```Output
<timestamp> ERROR PID <pid>: Exception calling "EndInvoke" with "1" argument(s): "Cannot process argument transformation on parameter 'inputData'. Cannot convert value "foo" to type "System.Int32". Error: "The input string 'foo' was not in a correct format.""
<timestamp> ERROR Failed to run process 'powershell': Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
<timestamp> ERROR Command: Resource 'powershell' [exit code 1] manifest description: PowerShell script execution failed
```

## Using input in a configuration document

You can pass input to a `WindowsPowerShellScript` instance inside a DSC configuration document, including
values from configuration parameters. The following configuration uses the [dsc config get][05]
command to pass a port number into the script:

```yaml
# check-port.dsc.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  port:
    type: int
    defaultValue: 8080
resources:
  - name: checkPort
    type: Microsoft.DSC.Transitional/WindowsPowerShellScript
    properties:
      getScript: |
        param($inputObj)
        Write-Information "Checking port $($inputObj.port)..."
        Test-NetConnection -ComputerName localhost -Port $inputObj.port |
          Select-Object -ExpandProperty TcpTestSucceeded
      input:
        port: "[parameters('port')]"
```

```powershell
dsc config get --file check-port.dsc.yaml
```

```yaml
executionInformation:
  # Elided for brevity
metadata:
  # Elided for brevity
results:
- executionInformation:
    duration: PT12.3218732S
  metadata:
    Microsoft.DSC:
      duration: PT12.3218732S
  name: checkPort
  type: Microsoft.DSC.Transitional/WindowsPowerShellScript
  result:
    actualState:
      output:
      - false
messages: []
hadErrors: false
```

<!-- Link definitions -->
[01]: ../index.md
[02]: https://learn.microsoft.com\powershell/module/microsoft.powershell.core/about/about_functions_advanced_parameters#parameter-declaration
[03]: https://learn.microsoft.com/powershell/module/microsoft.powershell.utility/convertfrom-json
[04]: https://learn.microsoft.com/powershell/module/microsoft.powershell.core/about/about_functions_advanced_parameters#type-conversion-of-parameter-values
[05]: ../../../../../../cli/config/get.md
