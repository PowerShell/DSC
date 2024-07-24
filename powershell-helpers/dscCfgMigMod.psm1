#region Main functions
function ConvertTo-DscJson
{
    <#
    .SYNOPSIS
        Convert a PowerShell DSC configuration document to DSC version 3 JSON format.
    
    .DESCRIPTION
        The function ConvertTo-DscJson converts a PowerShell DSC configuration document to DSC version 3 JSON format from a path.
    
    .PARAMETER Path
        The path to valid PowerShell DSC configuration document
    
    .EXAMPLE
        PS C:\> $configuration = @'
        Configuration TestResource {
            Import-DscResource -ModuleName TestResource
            Node localhost {
                TestResource 'Configure test resource' {
                    Ensure = 'Absent'
                    Name   = 'MyTestResource'
                }
            }
        }
        '@
        PS C:\> $Path = Join-Path -Path $env:TEMP -ChildPath 'configuration.ps1'
        PS C:\> $configuration | Out-File -FilePath $Path
        PS C:\> ConvertTo-DscJson -Path $Path

        Returns:
        {
            "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json",
            "resources": {
                "name": "TestResource",
                "type": "Microsoft.DSC/PowerShell",
                "properties": {
                    "resources": [
                        {
                            "name": "Configure test resource",
                            "type": "TestResource/TestResource",
                            "properties": {
                                "Name": "MyTestResource",
                                "Ensure": "Absent"
                            }
                        }
                    ]
                }
            }
        }
    
    .NOTES
        Tags: DSC, Migration, JSON
    #>
    [CmdletBinding()]
    Param 
    (
        [System.String]
        $Path
    )

    begin 
    {
        Write-Verbose ("Starting: {0}" -f $MyInvocation.MyCommand.Name)
    }

    process 
    {
        $inputObject = BuildConfigurationDocument -Path $Path
    }
    end
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
        return $inputObject
    }
}

function ConvertTo-DscYaml 
{
    <#
    .SYNOPSIS
        Convert a PowerShell DSC configuration document to DSC version 3 YAML format.
    
    .DESCRIPTION
        The function ConvertTo-DscYaml converts a PowerShell DSC configuration document to DSC version 3 YAML format from a path.
    
    .PARAMETER Path
        The path to valid PowerShell DSC configuration document
    
    .EXAMPLE
        PS C:\> $configuration = @'
        Configuration TestResource {
            Import-DscResource -ModuleName TestResource
            Node localhost {
                TestResource 'Configure test resource' {
                    Ensure = 'Absent'
                    Name   = 'MyTestResource'
                }
            }
        }
        '@
        PS C:\> $Path = Join-Path -Path $env:TEMP -ChildPath 'configuration.ps1'
        PS C:\> $configuration | Out-File -FilePath $Path
        PS C:\> ConvertTo-DscYaml -Path $Path

        Returns:
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
        resources:
        name: TestResource
        type: Microsoft.DSC/PowerShell
        properties:
            resources:
            - name: Configure test resource
              type: TestResource/TestResource
              properties:
                Name: MyTestResource
                Ensure: Absent
        
    .NOTES
        Tags: DSC, Migration, YAML
    #>
    [CmdletBinding()]
    Param 
    (
        [System.String]
        $Path
    )

    begin 
    {
        Write-Verbose ("Starting: {0}" -f $MyInvocation.MyCommand.Name)
    }

    process 
    {
        $inputObject = BuildConfigurationDocument -Path $Path -Format YAML
    }
    end
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
        return $inputObject
    }
}
#endRegion Main functions

#region Helper functions
function FindAndExtractConfigurationDocument
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Path
    )

    if (-not (TestPathExtension $Path))
    {
        return @{}
    }

    # Parse the abstract syntax tree to get all hash table values representing the configuration resources
    [System.Management.Automation.Language.Token[]] $tokens = $null
    [System.Management.Automation.Language.ParseError[]] $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$tokens, [ref]$errors)
    $configurations = $ast.FindAll({$args[0].GetType().Name -like 'HashtableAst'}, $true)

    # Create configuration document resource class (can be re-used)
    $configurationDocument = [DscConfigurationResource]::new()
    
    # Build simple regex
    $regex = [regex]::new('Configuration\s+(\w+)')
    $configValue = $regex.Matches($ast.Extent.Text).Value

    if (-not $configValue) 
    {
        return
    }

    $documentConfigurationName  = $configValue.TrimStart('Configuration').Trim(" ")

    # Start to build the outer basic format
    $configurationDocument.name = $documentConfigurationName
    $configurationDocument.type = 'Microsoft.DSC/PowerShell' # TODO: Add functions later to valid the adapter type

    # Bag to hold resources
    $resourceProps = [System.Collections.Generic.List[object]]::new()

    foreach ($configuration in $configurations) 
    {
        # Get parent configuration details
        $resourceName = ($configuration.Parent.CommandElements.Value | Select-Object -Last 1 )
        $resourceConfigurationName = ($configuration.Parent.CommandElements.Value | Select-Object -First 1)

        # Get module details 
        $module = Get-DscResource -Name $resourceConfigurationName -ErrorAction SilentlyContinue

        # Build the module
        $resource = [DscConfigurationResource]::new()
        $resource.properties = $configuration.SafeGetValue()
        $resource.name = $resourceName
        $resource.type = ("{0}/{1}" -f $module.ModuleName, $resourceConfigurationName)
        # TODO: Might have to change because it takes time. If there is only one Import-DscResource statement, we can simply RegEx it out, else use Get-DscResource
        # $document.ModuleName = $module.ModuleName

        Write-Verbose ("Adding document with data")
        Write-Verbose ($resource | ConvertTo-Json | Out-String)
        $resourceProps.Add($resource)
    }

    # Add all the resources
    $configurationDocument.properties = @{
        resources = $resourceProps
    }

    return $configurationDocument
}

function BuildConfigurationDocument
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Path,

        [ValidateSet('JSON', 'YAML')]
        [System.String]
        $Format = 'JSON'
    )

    $configurationDocument = [ordered]@{
        "`$schema" = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json" # TODO: Figure out how to extract latest document.json from schemas folder
        resources = FindAndExtractConfigurationDocument -Path $Path
    }

    switch ($Format)
    {
        "JSON" {
            $inputObject = ($configurationDocument | ConvertTo-Json -Depth 10)
        }
        "YAML" {
            if (TestYamlModule)
            {
                $inputObject = ($configurationDocument | ConvertTo-Yaml)
            }
            else 
            {
                $inputObject = @{}
            }
        }
        default {
            $inputObject = $configurationDocument
        }
    }

    return $inputObject
}

function TestPathExtension
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Path
    )

    $res = $true

    if (-not (Test-Path $Path))
    {
        $res = $false
    }

    if (([System.IO.Path]::GetExtension($Path) -ne ".ps1"))
    {
        $res = $false
    }

    return $res
}

function TestYamlModule 
{
    if (-not (Get-Command -Name 'ConvertTo-Yaml' -ErrorAction SilentlyContinue))
    {
        return $false 
    }

    return $true
}

function GetPowerShellPath
{
    param 
    (
        $Path
    )

    $knownPath = @(
        "$env:USERPROFILE\Documents\PowerShell\Modules",
        "$env:ProgramFiles\PowerShell\Modules",
        "$env:ProgramFiles\PowerShell\7\Modules"
    )

    foreach ($known in $knownPath)
    {
        if ($Path.StartsWith($known))
        {
            return $true
        }
    }

    return $false
}

function GetWindowsPowerShellPath
{
    param 
    (
        $Path
    )

    $knownPath = @(
        "$env:USERPROFILE\Documents\WindowsPowerShell\Modules",
        "$env:ProgramFiles\WindowsPowerShell\Modules",
        "$env:SystemRoot\System32\WindowsPowerShell\v1.0\Modules"
    )

    foreach ($known in $knownPath)
    {
        if ($Path.StartsWith($known))
        {
            return $true
        }
    }

    return $false
}

function ResolvePowerShellPath
{
    [CmdletBinding()]
    Param
    (
        [System.String]
        $Path
    )

    if (-not (Test-Path $Path))
    {
        return
    }

    if (([System.IO.Path]::GetExtension($Path) -ne ".psm1"))
    {
        return
    }

    if (GetPowerShellPath -Path $Path)
    {
        return "Microsoft.DSC/PowerShell"
    }

    if (GetWindowsPowerShellPath -Path $Path)
    {
        return "Microsoft.Windows/WindowsPowerShell"
    }

    return $null # TODO: Or default Microsoft.DSC/PowerShell
}

#endRegion Helper functions

#region Classes
class DscConfigurationResource
{
    [string] $name
    [string] $type
    [hashtable] $properties
}
#endRegion classes