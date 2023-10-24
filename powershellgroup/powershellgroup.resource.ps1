# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [ValidateSet('List','Get','Set','Test')]
    $Operation = 'List',
    [Switch]
    $WinPS = $false,
    [Parameter(ValueFromPipeline)]
    $stdinput
)

$ProgressPreference = 'Ignore'
$WarningPreference = 'Ignore'
$VerbosePreference = 'Ignore'
$script:ResourceCache = @{}

function RefreshCache
{
    $script:ResourceCache = @{}

    $DscResources = Get-DscResource
    
    foreach ($r in $DscResources)
    {
        $moduleName = "";
        if ($r.ModuleName) { $moduleName = $r.ModuleName }
        elseif ($r.ParentPath) { $moduleName = Split-Path $r.ParentPath | Split-Path | Split-Path -Leaf }

        $fullResourceTypeName = "$moduleName/$($r.ResourceType)"
        $script:ResourceCache[$fullResourceTypeName] = $r
    }
}

if (($PSVersionTable.PSVersion.Major -ge 7) -and ($PSVersionTable.PSVersion.Minor -ge 4))
{
    throw "PowerShell 7.4 is not currently supported by PowerShellGroup resource; please use PS 7.3.  Tracking issue: https://github.com/PowerShell/DSC/issues/128"
}

$DscModule = Get-Module -Name PSDesiredStateConfiguration -ListAvailable |
    Sort-Object -Property Version -Descending |
    Select-Object -First 1

if ($null -eq $DscModule)
{
    Write-Error "Could not find and import the PSDesiredStateConfiguration module."
    # Missing module is okay for listing resources
    if ($Operation -eq 'List') { exit 0 }

    exit 1
}

Import-Module $DscModule

if ($Operation -eq 'List')
{

    RefreshCache
    #TODO: following should be added to debug stream of every operation
    #$m = gmo PSDesiredStateConfiguration
    #$r += @{"DebugInfo"=@{"ModuleVersion"=$m.Version.ToString();"ModulePath"=$m.Path;"PSVersion"=$PSVersionTable.PSVersion.ToString();"PSPath"=$PSHome}}
    #$r[0] | ConvertTo-Json -Compress -Depth 3
    foreach ($r in $script:ResourceCache.keys)
    {
        if ($script:ResourceCache[$r].ImplementedAs -eq "Binary")
        {
            continue
        }

        $version_string = "";
        if ($script:ResourceCache[$r].Version) { $version_string = $script:ResourceCache[$r].Version.ToString() }
        $author_string = "";
        if ($script:ResourceCache[$r].author) { $author_string = $script:ResourceCache[$r].CompanyName.ToString() }
        $moduleName = "";
        if ($script:ResourceCache[$r].ModuleName) { $moduleName = $script:ResourceCache[$r].ModuleName }
        elseif ($script:ResourceCache[$r].ParentPath) { $moduleName = Split-Path $script:ResourceCache[$r].ParentPath | Split-Path | Split-Path -Leaf }

        $propertyList = @()
        foreach ($p in $script:ResourceCache[$r].Properties)
        {
            if ($p.Name)
            {
                $propertyList += $p.Name
            }
        }

        $fullResourceTypeName = "$moduleName/$($script:ResourceCache[$r].ResourceType)"
       if ($WinPS) {$requiresString = "DSC/WindowsPowerShellGroup"} else {$requiresString = "DSC/PowerShellGroup"}

        $z = [pscustomobject]@{
            type = $fullResourceTypeName;
            version = $version_string;
            path = $script:ResourceCache[$r].Path;
            directory = $script:ResourceCache[$r].ParentPath;
            implementedAs = $script:ResourceCache[$r].ImplementationDetail;
            author = $author_string;
            properties = $propertyList;
            requires = $requiresString
        }

        $z | ConvertTo-Json -Compress
    }
}
elseif ($Operation -eq 'Get')
{
    $inputobj_pscustomobj = $null
    if ($stdinput)
    {
        $inputobj_pscustomobj = $stdinput | ConvertFrom-Json
    }
    
    $result = @()

    RefreshCache

    if ($inputobj_pscustomobj.resources) # we are processing a config batch
    {
        foreach($r in $inputobj_pscustomobj.resources)
        {
            #Write-Output $r.type
            $cachedResourceInfo = $script:ResourceCache[$r.type]
            if ($cachedResourceInfo)
            {
                $inputht = @{}
                $typeparts = $r.type -split "/"
                $ModuleName = $typeparts[0]
                $ResourceTypeName = $typeparts[1]

                if ($r.ImplementationDetail -eq 'ScriptBased') {
                    Import-Module -Scope Local -Name $r.path -Force -ErrorAction stop
                    $validParams = (Get-Command -Module $r.ResourceType -Name 'Get-TargetResource').Parameters.Keys
                    $r.properties.psobject.properties | ForEach-Object {
                        if ($validParams -notcontains $_.Name) {
                            $r.properties.psobject.properties.Remove($_.Name)
                        }
                    }
                }
                
                $r.properties.psobject.properties | %{ $inputht[$_.Name] = $_.Value }
                $e = $null
                $op_result = Invoke-DscResource -Method Get -ModuleName $ModuleName -Name $ResourceTypeName -Property $inputht -ErrorVariable e
                if ($e)
                {
                    # By this point Invoke-DscResource already wrote error message to stderr stream,
                    # so we just need to signal error to the caller by non-zero exit code.
                    exit 1
                }
                $result += $op_result
            }
            else
            {
                $errmsg = "Can not find type " + $r.type + "; please ensure that Get-DscResource returns this resource type"
                Write-Error $errmsg
                exit 1
            }
        }
    }
    else # we are processing an individual resource call
    {
        $cachedResourceInfo = $script:ResourceCache[$inputobj_pscustomobj.type]
        if ($cachedResourceInfo)
        {
            $inputht = @{}
            $ResourceTypeName = ($inputobj_pscustomobj.type -split "/")[1]

            if ($r.ImplementationDetail -eq 'ScriptBased') {
                Import-Module -Scope Local -Name $r.path -Force -ErrorAction stop
                $validParams = (Get-Command -Module $r.ResourceType -Name 'Get-TargetResource').Parameters.Keys
                $inputobj_pscustomobj.psobject.properties | ForEach-Object {
                    if ($validParams -notcontains $_.Name) {
                        $inputobj_pscustomobj.psobject.properties.Remove($_.Name)
                    }
                }
            }
            
            $inputobj_pscustomobj.psobject.properties | %{ 
                if ($_.Name -ne "type")
                {
                    $inputht[$_.Name] = $_.Value
                }
            }
            $e = $null
            $op_result = Invoke-DscResource -Method Get -Name $ResourceTypeName -Property $inputht -ErrorVariable e
            if ($e)
            {
                # By this point Invoke-DscResource already wrote error message to stderr stream,
                # so we just need to signal error to the caller by non-zero exit code.
                exit 1
            }
            $result = $op_result
        }
        else
        {
            $errmsg = "Can not find type " + $inputobj_pscustomobj.type + "; please ensure that Get-DscResource returns this resource type"
            Write-Error $errmsg
            exit 1
        }
    }

    $result | ConvertTo-Json -EnumsAsStrings
}
elseif ($Operation -eq 'Set')
{
    $inputobj_pscustomobj = $null
    if ($stdinput)
    {
        $inputobj_pscustomobj = $stdinput | ConvertFrom-Json
    }

    $result = @()

    RefreshCache

    if ($inputobj_pscustomobj.resources) # we are processing a config batch
    {
        foreach($r in $inputobj_pscustomobj.resources)
        {
            #Write-Output $r.type
            $cachedResourceInfo = $script:ResourceCache[$r.type]
            if ($cachedResourceInfo)
            {
                $inputht = @{}
                $ResourceTypeName = ($r.type -split "/")[1]
                $r.properties.psobject.properties | %{ $inputht[$_.Name] = $_.Value }
                $e = $null
                $op_result = Invoke-DscResource -Method Set -Name $ResourceTypeName -Property $inputht -ErrorVariable e
                if ($e)
                {
                    # By this point Invoke-DscResource already wrote error message to stderr stream,
                    # so we just need to signal error to the caller by non-zero exit code.
                    exit 1
                }
                $result += $op_result
            }
            else
            {
                $errmsg = "Can not find type " + $r.type + "; please ensure that Get-DscResource returns this resource type"
                Write-Error $errmsg
                exit 1
            }
        }
    }
    else # we are processing an individual resource call
    {
        $cachedResourceInfo = $script:ResourceCache[$inputobj_pscustomobj.type]
        if ($cachedResourceInfo)
        {
            $inputht = @{}
            $ResourceTypeName = ($inputobj_pscustomobj.type -split "/")[1]
            $inputobj_pscustomobj.psobject.properties | %{ 
                if ($_.Name -ne "type")
                {
                    $inputht[$_.Name] = $_.Value
                }
            }
            $e = $null
            $op_result = Invoke-DscResource -Method Set -Name $ResourceTypeName -Property $inputht -ErrorVariable e
            if ($e)
            {
                # By this point Invoke-DscResource already wrote error message to stderr stream,
                # so we just need to signal error to the caller by non-zero exit code.
                exit 1
            }
            $result = $op_result
        }
        else
        {
            $errmsg = "Can not find type " + $inputobj_pscustomobj.type + "; please ensure that Get-DscResource returns this resource type"
            Write-Error $errmsg
            exit 1
        }
    }

    $result | ConvertTo-Json
}
elseif ($Operation -eq 'Test')
{
    $inputobj_pscustomobj = $null
    if ($stdinput)
    {
        $inputobj_pscustomobj = $stdinput | ConvertFrom-Json
    }

    $result = @()

    RefreshCache

    if ($inputobj_pscustomobj.resources) # we are processing a config batch
    {
        foreach($r in $inputobj_pscustomobj.resources)
        {
            #Write-Output $r.type
            $cachedResourceInfo = $script:ResourceCache[$r.type]
            if ($cachedResourceInfo)
            {
                $inputht = @{}
                $ResourceTypeName = ($r.type -split "/")[1]
                $r.properties.psobject.properties | %{ $inputht[$_.Name] = $_.Value }
                $e = $null
                $op_result = Invoke-DscResource -Method Test -Name $ResourceTypeName -Property $inputht -ErrorVariable e
                if ($e)
                {
                    # By this point Invoke-DscResource already wrote error message to stderr stream,
                    # so we just need to signal error to the caller by non-zero exit code.
                    exit 1
                }
                $result += $op_result
            }
            else
            {
                $errmsg = "Can not find type " + $r.type + "; please ensure that Get-DscResource returns this resource type"
                Write-Error $errmsg
                exit 1
            }
        }
    }
    else # we are processing an individual resource call
    {
        $cachedResourceInfo = $script:ResourceCache[$inputobj_pscustomobj.type]
        if ($cachedResourceInfo)
        {
            $inputht = @{}
            $ResourceTypeName = ($inputobj_pscustomobj.type -split "/")[1]
            $inputobj_pscustomobj.psobject.properties | %{ 
                if ($_.Name -ne "type")
                {
                    $inputht[$_.Name] = $_.Value
                }
            }
            $e = $null
            $op_result = Invoke-DscResource -Method Get -Name $ResourceTypeName -Property $inputht -ErrorVariable e
            if ($e)
            {
                # By this point Invoke-DscResource already wrote error message to stderr stream,
                # so we just need to signal error to the caller by non-zero exit code.
                exit 1
            }
            $result = $op_result
        }
        else
        {
            $errmsg = "Can not find type " + $inputobj_pscustomobj.type + "; please ensure that Get-DscResource returns this resource type"
            Write-Error $errmsg
            exit 1
        }
    }

    $result | ConvertTo-Json
}
else
{
    "ERROR: Unsupported operation requested from powershellgroup.resource.ps1"
}
