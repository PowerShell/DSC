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

if ($Operation -eq 'List')
{
    $DscResources= Get-DscResource
    #$m = gmo PSDesiredStateConfiguration
    #$r += @{"DebugInfo"=@{"ModuleVersion"=$m.Version.ToString();"ModulePath"=$m.Path;"PSVersion"=$PSVersionTable.PSVersion.ToString();"PSPath"=$PSHome}}
    #$r[0] | ConvertTo-Json -Compress -Depth 3

    foreach ($r in $DscResources)
    {
        $version_string = "";
        if ($r.Version) { $version_string = $r.Version.ToString() }
        $author_string = "";
        if ($r.author) { $author_string = $r.CompanyName.ToString() }

        $moduleName = "";
        if ($r.ModuleName) { $moduleName = $r.ModuleName }
        elseif ($r.ParentPath) { $moduleName = Split-Path $r.ParentPath | Split-Path | Split-Path -Leaf }

        $propertyList = @()
        if ($r.Properties)
        {
            $propertyList = @($r.Properties.Name)
        }

        $z = [pscustomobject]@{
            type = "$moduleName/$($r.ResourceType)";
            version = $version_string;
            path = $r.Path;
            directory = $r.ParentPath;
            implementedAs = $r.ImplementationDetail;
            author = $author_string;
            properties = $propertyList;
            requires = "DSC/PowerShellGroup"
        }

        $z | ConvertTo-Json -Compress
    }
}
elseif ($Operation -eq 'Get')
{
    $inputobj_pscustomobj = $stdinput | ConvertFrom-Json

    $result = @()
    if ($inputobj_pscustomobj.resources) # we are processing a config batch
    {
        foreach($r in $inputobj_pscustomobj.resources)
        {
            $inputht = @{}
            $ResourceTypeName = ($r.type -split "/")[1]
            $r.properties.psobject.properties | %{ $inputht[$_.Name] = $_.Value }
            $result += Invoke-DscResource -Method Get -Name $ResourceTypeName -Property $inputht
        }
    }

    $result | ConvertTo-Json
}
else
{
    "ERROR: Unsupported operation."
}