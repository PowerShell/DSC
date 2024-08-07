function Build-DscConfigurationDocument
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Path,

        [ValidateSet('JSON', 'YAML', 'Default')]
        [System.String]
        $Format = 'JSON'
    )

    $configurationDocument = [ordered]@{
        "`$schema" = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json"
        resources = Export-DscConfigurationDocument -Path $Path
    }

    switch ($Format)
    {
        "JSON" {
            $inputObject = ($configurationDocument | ConvertTo-Json -Depth 10)
        }
        "YAML" {
            if (Test-YamlModule)
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