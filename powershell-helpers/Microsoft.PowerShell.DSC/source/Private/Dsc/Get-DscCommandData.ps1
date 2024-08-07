function Get-DscCommandData 
{
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $false)]
        [System.String]
        $CommandName,

        [Parameter(Mandatory = $false)]
        [System.Management.Automation.SwitchParameter]
        $IncludeProperties,

        [Parameter(Mandatory = $false)]
        [System.String]
        $ResourceName,

        [Parameter(Mandatory = $false)]
        [ValidateSet('Get', 'Set', 'Test')]
        [System.String]
        $Operation = 'Get'
    )

    $exeLocation = Resolve-DscExe -ErrorAction SilentlyContinue
    if ($exeLocation)
    {
        $files = Get-ChildItem -Path (Split-Path -Path $exeLocation -Parent) -Filter '*.dsc.resource.json'
        $resources = $files | ForEach-Object {
            (Get-Content $_ | ConvertFrom-Json -ErrorAction SilentlyContinue).type
            
        }
    }

    $cmdData = @{
        'Example' = @{
            '1.0' = @{
                SubCommand = 'resource list'
            }
        }
        'ExampleSnippet' = @{
            'Microsoft.Windows/Registry' = @{
                Get = @{ keyPath = "HKCU\Microsoft"}
                Set = @{ keyPath = "HKCU\1"; valueName = "Desired State"; valueData = @{"String" = "Configuration"} }
                Test = @{ keyPath = "HKCU\Microsoft"}
            }
        }
        'Get-DscResourceCommand' = @{
            'preview8' = @{
                SubCommand = 'resource get'
                Resources = $resources
            }
        }
        'Set-DscResourceCommand' = @{
            'preview8' = @{
                SubCommand = 'resource set'
                Resources = $resources
            }
        }
        'Test-DscResourceCommand' = @{
            'preview8' = @{
                SubCommand = 'resource test'
                Resources = $resources
            }
        }
    }

    # TODO: Add possible version info
    $keyData = $cmdData.$CommandName.Keys

    if ($null -eq $keyData)
    {
        Throw "Cannot find data entry for '$CommandName'. Please make sure the $($MyInvocation.MyCommand.Name) is up to date with data."
    }

    $result = ($cmdData.$CommandName | Where-Object keys -eq $keyData).Values

    if ($IncludeProperties)
    {
        if (-not $ResourceName)
        {
            Throw "When specifying '-IncludeProperties', you have to include the '-ResourceName' parameter also."
        }

        if (-not (Test-DscResourceName -ResourceName $ResourceName -Resources $resources))
        { 
            $result.Add('properties', @{}) 
        }
        else 
        {
            # get schema details
            $properties = Get-DscResourceSchemaProperty -ResourceName $ResourceName -Operation $Operation

            # add them as properties
            Write-Verbose -Message ("Including propery data: $($properties | ConvertTo-Json | Out-String)")
            $result.Add('properties', $properties)

            # get example snippet if available
            $snippet = $cmdData.ExampleSnippet.$ResourceName.$Operation
            Write-Verbose -Message ("Including example snippet data: $($snippet | ConvertTo-Json | Out-String)")
            $result.Add('exampleSnippet', $snippet)
        }
    }

    Write-Verbose -Message "Selected data for '$CommandName'"
    return $result
}
