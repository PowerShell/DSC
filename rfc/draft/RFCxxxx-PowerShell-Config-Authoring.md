---
RFC:          RFCNNNN # WG will set the number after submission
Author:       @SteveL-MSFT
Sponsor:      null    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      0.1     # <Major>.<Minor>
Area:         Configuration
CommentsDue:  null    # <Date for submitting comments to current draft (minimum 1 month)>
---

# PowerShell script authoring for Desired State Configuration

Enable PowerShell script authors to create an imperative script that can be used to generate a DSC configuration document.
This allows script authors to use the full power of PowerShell to create a configuration document,
while still allowing the configuration document to be used in a declarative manner.

## Motivation

> As an IT administrator,
> I want to author a complex configuration with loops, conditionals, and other imperative constructs,
> so that I can generate a DSC configuration document using context from my imperative script.

IT Professionals typically use a higher level domain specific language (DSL) to author configurations.
Examples include PowerShell DSC, Bicep, and Terraform.
These DSLs allow for imperative constructs such as loops and conditionals to be used to generate a declarative configuration document.
This allows for authoring complex configurations that can then be used to generate multiple declarative configuration documents for different environments.

Within PowerShell, this was previously accomplished by using the `configuration` keyword in a PowerShell script which would generate a
legacy DSC mof file.
However, this approach is no longer viable:

- The `mof` file format is not widely adopted and therefore not used by the new DSC engine.
- The new configuration document format supports expressions which is not supported by `mof`.

## Proposed experience

There are two main aspects to the proposed experience:

- Intellisense support for authoring the different components that compose a configuration document.
- Transpiling PowerShell script to a DSC expression.

_NOTE_: Bi-directional transpiling of the configuration document to/from PowerShell script is not in scope for this RFC.
Technical decisions made should not preclude the ability to transpile a configuration document back to PowerShell script in the future.
There is no expectation that round-trip transpiling would retain full fidelity of the original PowerShell script, but it should be possible to transpile a configuration document back to PowerShell script that would generate the same functional configuration document.

Creating a DSC configuration document:

```powershell
Import-Module Microsoft.DesiredStateConfiguration

$config = New-DscConfiguration
$config.Metadata = @{
    Name = 'MyConfiguration'
    Version = '1.0.0'
    Author = 'SteveL-MSFT'
}

# cmdlets provided to create consistent discovery experience
# the `parameters` member is a collection of `[Dsc.Parameter]` objects
$config.Parameters += New-DscParameter -Name 'ComputerName' -Type 'string' -Required

# users can also use the types directly to create a parameter
$config.Parameters += [DSC.Parameter]@{
    Name = 'Environment'
    Type = [Dsc.DataType]::String
    DefaultValue = 'Production'
}

# The `-Type` would allow for intellisense performing the equivalent to `dsc resource list` on statically cached
# resources found during module import.
$echoResource = New-DscResource -Name 'My echo' -Type 'Microsoft.DSC.Debug/Echo'

# The resulting `[DSC.Resource]` object would have a `Properties` property that would allow for intellisense to provide the available properties for the resource.
$echoResource.Properties.Output = 'Hello World'

$echoResource2 = New-DscResource -Name 'My echo 2' -Type 'Microsoft.DSC.Debug/Echo'

# Here we use a scriptblock to generate a DSC expression
$echoResource2.Properties.Output = {
    $config.Parameters['Environment'] + ' ' + $config.Parameters['ComputerName']
}

# Handle dependencies between resources by using the `DependsOn` property of the `[Dsc.Resource]` object.
$echoResource2.DependsOn += $echoResource

# The `Resources` property is a collection of `[Dsc.Resource]` objects
$config.Resources += $echoResource
$config.Resources += $echoResource2

$config.Export('./MyConfiguration.dsc.json')

# alternatively using cmdlet
$config | Export-DscConfiguration -Path './MyConfiguration.dsc.json'
```

This would generate the following configuration document:

```json
{
  "Metadata": {
    "Name": "MyConfiguration",
    "Version": "1.0.0",
    "Author": "SteveL-MSFT"
  },
  "Parameters": [
    {
      "Name": "ComputerName",
      "Type": "string",
      "Required": true
    },
    {
      "Name": "Environment",
      "Type": "string",
      "DefaultValue": "Production"
    }
  ],
  "Resources": [
    {
      "Name": "My echo",
      "Type": "Microsoft.DSC.Debug/Echo",
      "Properties": {
        "Output": "Hello World"
      }
    },
    {
      "Name": "My echo 2",
      "Type": "Microsoft.DSC.Debug/Echo",
      "Properties": {
        "Output": "[concat(parameters('Environment'), ' ', parameters('ComputerName'))]"
      },
      "DependsOn": [
        "[resourceId('Microsoft.DSC.Debug/Echo', 'My echo')]"
      ]
    }
  ]
}
```

## Specification

### Configuration document authoring

- Provide both cmdlets and types to create a configuration document to aid with discovery and intellisense
- `[Dsc.Configuration]` type has `Export()` method
- Scriptblocks are transpiled to `[Dsc.Expression]` objects
  - The transpiler should allow for idiomatic PowerShell script where multiple types of statements can result in the same expression
  - Functions in DSC that don't exist in PowerShell should be presented as cmdlets
  - Error is returned during parsing if a scriptblock cannot be transpiled to a DSC expression

## Alternate Proposals and Considerations

<!--
    Include any alternate proposals and notes for the RFC in this section.
-->

## Related work items

<!--
    Include any relevant GitHub issues, discussions, and pull requests as unordered list items
    in this section. If the work item title doesn't clearly indicate how it relates to this
    RFC, add a short summary statement after the work item.

    For example:

    - #123 - Indicates the need for and prior conversation around discovering DSC resources from
      remote registries.
-->
