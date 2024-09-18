function ConvertTo-DscJson
{
    <#
    .SYNOPSIS
        Convert DSC Configuration (v1/v2) Document to JSON
    
    .DESCRIPTION
        The function ConvertTo-DscJson converts a DSC Configuration Document (v1/v2) to JSON
    
    .PARAMETER Path
        The file path to a valid DSC Configuration Document
    
    .EXAMPLE
        PS C:\> $path = 'myConfig.ps1'
        PS C:\> ConvertTo-DscJson -Path $path

    .INPUTS
        Input a valid DSC Configuration Document.

        configuration MyConfiguration {
            Import-DscResource -ModuleName PSDesiredStateConfiguration 
            Node localhost
            {
                Environment CreatePathEnvironmentVariable
                {
                    Name = 'TestPathEnvironmentVariable'
                    Value = 'TestValue'
                    Ensure = 'Present'
                    Path = $true
                    Target = @('Process')
                }
            }
        }
        
    .OUTPUTS
        Returns a JSON string.

        {
            "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json",
            "resources": {
                "name": "MyConfiguration \r\n    node",
                "type": "Microsoft.DSC/PowerShell",
                "properties": {
                "resources": [
                    {
                    "name": "CreatePathEnvironmentVariable",
                    "type": "PSDscResources/Environment",
                    "properties": {
                        "Value": "TestValue",
                        "Path": true,
                        "Name": "TestPathEnvironmentVariable",
                        "Ensure": "Present",
                        "Target": [
                                    "Process"
                                ]
                            }
                        }
                    ]
                }
            }
        } 
    #>
    [CmdletBinding()]
    [OutputType([System.String])]
    Param 
    (
        [Parameter(ValueFromPipeline = $true)]
        [System.String]
        $Path
    )

    begin 
    {
        $commandName = $MyInvocation.MyCommand.Name 
        Write-Verbose ("Starting: {0}" -f $commandName)
    }

    process 
    {
        $inputObject = Build-DscConfigurationDocument -Path $Path
    }
    end
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
        return $inputObject
    }
}