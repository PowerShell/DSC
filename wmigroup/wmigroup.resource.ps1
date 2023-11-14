# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [ValidateSet('List','Get','Set','Test')]
    $Operation = 'List',
    [Parameter(ValueFromPipeline)]
    $stdinput
)

$ProgressPreference = 'Ignore'
$WarningPreference = 'Ignore'
$VerbosePreference = 'Ignore'

if ($Operation -eq 'List')
{
    $clases = Get-CimClass
    
    foreach ($r in $clases)
    {
        $version_string = "";
        $author_string = "";
        $moduleName = "";

        $propertyList = @()
        foreach ($p in $r.CimClassProperties)
        {
            if ($p.Name)
            {
                $propertyList += $p.Name
            }
        }

        $namespace = $r.CimSystemProperties.Namespace.ToLower().Replace('/','\')
        $classname = $r.CimSystemProperties.ClassName
        $fullResourceTypeName = "$namespace/$classname"
        $requiresString = "DSC/WMIGroup"

        $z = [pscustomobject]@{
            type = $fullResourceTypeName;
            version = $version_string;
            path = "";
            directory = "";
            implementedAs = "";
            author = $author_string;
            properties = $propertyList;
            requires = $requiresString
        }

        $z | ConvertTo-Json -Compress
    }
}
else
{
    Write-Error "ERROR: Unsupported operation requested from wmigroup.resource.ps1"
}