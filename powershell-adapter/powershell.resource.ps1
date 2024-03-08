# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [ValidateSet('List','Get','Set','Test','Export','Validate')]
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

function IsConfiguration($obj) {
    if ($null -ne $obj.metadata -and $null -ne $obj.metadata.'Microsoft.DSC' -and $obj.metadata.'Microsoft.DSC'.context -eq 'Configuration') {
        return $true
    }

    return $false
}

if (($PSVersionTable.PSVersion.Major -eq 7) -and ($PSVersionTable.PSVersion.Minor -eq 4) `
   -and ($PSVersionTable.PSVersion.PreReleaseLabel.StartsWith("preview")))
{
    throw "PowerShell 7.4-previews are not supported by PowerShell adapter resource; please use PS 7.4.0-rc.1 or newer."
}

$inputobj_pscustomobj = $null
if ($stdinput)
{
    $inputobj_pscustomobj = $stdinput | ConvertFrom-Json
    $new_psmodulepath = $inputobj_pscustomobj.psmodulepath
    if ($new_psmodulepath)
    {
        $env:PSModulePath = $ExecutionContext.InvokeCommand.ExpandString($new_psmodulepath)
    }
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

Import-Module $DscModule -DisableNameChecking

if ($Operation -eq 'List')
{
    $DscResources= Get-DscResource
    #TODO: following should be added to debug stream of every operation
    #$m = gmo PSDesiredStateConfiguration
    #$r += @{"DebugInfo"=@{"ModuleVersion"=$m.Version.ToString();"ModulePath"=$m.Path;"PSVersion"=$PSVersionTable.PSVersion.ToString();"PSPath"=$PSHome}}
    #$r[0] | ConvertTo-Json -Compress -Depth 3
    foreach ($r in $DscResources)
    {
        if ($r.ImplementedAs -eq "Binary")
        {
            continue
        }

        $version_string = "";
        if ($r.Version) { $version_string = $r.Version.ToString() }
        $author_string = "";
        if ($r.author) { $author_string = $r.CompanyName.ToString() }
        $moduleName = "";
        if ($r.ModuleName) { $moduleName = $r.ModuleName }
        elseif ($r.ParentPath) { $moduleName = Split-Path $r.ParentPath | Split-Path | Split-Path -Leaf }

        $propertyList = @()
        foreach ($p in $r.Properties)
        {
            if ($p.Name)
            {
                $propertyList += $p.Name
            }
        }

        $fullResourceTypeName = "$moduleName/$($r.ResourceType)"
        $script:ResourceCache[$fullResourceTypeName] = $r
        if ($WinPS) {$requiresString = "Microsoft.Windows/WindowsPowerShell"} else {$requiresString = "Microsoft.DSC/PowerShell"}

        $t = [Type]$r.ResourceType
        $exportMethod = $t.GetMethod('Export')

        $capabilities = @('Get', 'Set', 'Test')
        if ($null -ne $exportMethod) {
            $capabilities += 'Export'
        }

        $z = [pscustomobject]@{
            type = $fullResourceTypeName;
            kind = 'Resource';
            version = $version_string;
            capabilities = $capabilities;
            path = $r.Path;
            directory = $r.ParentPath;
            implementedAs = $r.ImplementationDetail;
            author = $author_string;
            properties = $propertyList;
            requires = $requiresString
        }

        $z | ConvertTo-Json -Compress
    }
}
elseif ($Operation -eq 'Get')
{
    $result = @()

    RefreshCache

    if (IsConfiguration $inputobj_pscustomobj) # we are processing a config batch
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
            $inputobj_pscustomobj.psobject.properties | %{
                if ($_.Name -ne "type")
                {
                    $inputht[$_.Name] = $_.Value
                }
            }
            $e = $null
            $op_result = Invoke-DscResource -Method Get -Name $ResourceTypeName -Property $inputht -ErrorVariable e -WarningAction SilentlyContinue
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
    $result = @()

    RefreshCache

    if (IsConfiguration $inputobj_pscustomobj) # we are processing a config batch
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
    $result = @()

    RefreshCache

    if (IsConfiguration $inputobj_pscustomobj) # we are processing a config batch
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
            $op_result = Invoke-DscResource -Method Test -Name $ResourceTypeName -Property $inputht -ErrorVariable e
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
elseif ($Operation -eq 'Export')
{
    $result = @()

    RefreshCache

    if (IsConfiguration $inputobj_pscustomobj) # we are processing a config batch
    {
        foreach($r in $inputobj_pscustomobj.resources)
        {
            $cachedResourceInfo = $script:ResourceCache[$r.type]
            if ($cachedResourceInfo)
            {
                $path = $cachedResourceInfo.Path # for class-based resources - this is path to psd1 of their defining module

                $typeparts = $r.type -split "/"
                $ResourceTypeName = $typeparts[1]

                $scriptBody = "using module '$path'"
                $script = [ScriptBlock]::Create($scriptBody)
                . $script

                $t = [Type]$ResourceTypeName
                $method = $t.GetMethod('Export')
                $resultArray = $method.Invoke($null,$null)
                foreach ($instance in $resultArray)
                {
                    $instance | ConvertTo-Json -Compress | Write-Output
                }
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
            $path = $cachedResourceInfo.Path # for class-based resources - this is path to psd1 of their defining module

            $typeparts = $inputobj_pscustomobj.type -split "/"
            $ResourceTypeName = $typeparts[1]

            $scriptBody = "using module '$path'"
            $script = [ScriptBlock]::Create($scriptBody)
            . $script

            $t = [Type]$ResourceTypeName
            $method = $t.GetMethod('Export')
            $resultArray = $method.Invoke($null,$null)
            foreach ($instance in $resultArray)
            {
                $instance | ConvertTo-Json -Compress | Write-Output
            }
        }
        else
        {
            $errmsg = "Can not find type " + $inputobj_pscustomobj.type + "; please ensure that Get-DscResource returns this resource type"
            Write-Error $errmsg
            exit 1
        }
    }
}
elseif ($Operation -eq 'Validate')
{
    # TODO: this is placeholder
    @{ valid = $true } | ConvertTo-Json
}
else
{
    "ERROR: Unsupported operation requested from powershell.resource.ps1"
}