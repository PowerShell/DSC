function Get-DscResourceSchemaProperty
{
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $ResourceName,

        [Parameter(Mandatory = $false)]
        [ValidateSet('Get', 'Set', 'Test')]
        [System.String]
        $Operation = 'Get'
    )

    $resource = $ResourceName.Split("/")[-1]
    $resourceManifestFile = Join-Path (Split-Path -Path $(Resolve-DscExe) -Parent) -ChildPath "$resource.dsc.resource.json"

    $result = @{}

    if (Test-Path $resourceManifestFile)
    {
        $content = Get-Content $resourceManifestFile | ConvertFrom-Json

        # only return resource kind types
        $knownTypes = @('group', 'adapter')
        if ($content.type -notin $knownTypes)
        {
            if (-not ($content.schema))
            {
                # content does not have schema
                # TODO: Check if this can be the case
                return $result
            }

            $fileExe = $content.schema.command.executable

            if ($fileExe)
            {
                $inputObject = (& $fileExe $content.schema.command.args | ConvertFrom-Json -ErrorAction SilentlyContinue)
            }

            if ($content.schema.embedded)
            {
                $inputObject = $content.schema.embedded
            }

            $result = [System.Collections.Generic.List[hashtable]]::new()
            switch ($Operation)
            {
                'Get'
                {
                    $inputObject.required | ForEach-Object {
                        $add = @{
                            $_ = "<$_>"
                        }
    
                        [void]$result.Add($add)
                    }  
                }
                'Set'
                {
                    $add = @{}
                    ($inputObject.properties | Get-Member -MemberType NoteProperty) | ForEach-Object {
                        if (-not $add["$_"]) 
                        {
                            $add += @{
                                $_.Name = "<$($_.Name)>"
                            }
                        }  
                    }
                    [void]$result.Add($add)
                }
                'Test' 
                {
                    $inputObject.required | ForEach-Object {
                        $add = @{
                            $_ = "<$_>"
                        }
    
                        [void]$result.Add($add)
                    }  
                }
                Default { $result.Add(@{}) }
            }   
        }

        return $result
    }
}
