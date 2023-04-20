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
    $r = Get-DscResource
    $m = gmo PSDesiredStateConfiguration
    $r += @{"DebugInfo"=@{"ModuleVersion"=$m.Version.ToString();"ModulePath"=$m.Path;"PSVersion"=$PSVersionTable.PSVersion.ToString();"PSPath"=$PSHome}}
    $r | ConvertTo-Json -Depth 3
}
elseif ($Operation -eq 'Get')
{
    $inputobj_pscustomobj = $stdinput | ConvertFrom-Json
    
    $inputht = @{}
    $ResourceTypeName = ""

    $inputobj_pscustomobj.psobject.properties | Foreach { 
        if ($_.Name -eq "Resource")
        {
            $ResourceTypeName = $_.Value
        }
        else
        {
       ` $inputht[$_.Name] = $_.Value
        }
    }
    $result = Invoke-DscResource -Method Get -Name $ResourceTypeName -Property $inputht
    $result | ConvertTo-Json
}
else
{
    "ERROR: Unsupported operation."
}