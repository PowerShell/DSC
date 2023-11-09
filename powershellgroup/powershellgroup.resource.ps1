# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [ValidateSet('List', 'Get', 'Set', 'Test')]
    $Operation = 'List',
    [Switch]
    $WinPS = $false,
    [Parameter(ValueFromPipeline)]
    $stdin
)

$ProgressPreference = 'Ignore'
$WarningPreference = 'Ignore'
$VerbosePreference = 'Ignore'

$DscModule = Get-Module -Name PSDesiredStateConfiguration -ListAvailable |
Sort-Object -Property Version -Descending |
Select-Object -First 1

if ($null -EQ $DscModule) {
    Write-Error 'Could not find and import the PSDesiredStateConfiguration module.'
    # Missing module is okay for listing resources
    if ($Operation -EQ 'List') { exit 0 }

    exit 1
}
Import-Module $DscModule

# cache the results of Get-DscResource
[resourceCache[]]$resourceCache = @()
$DscResources = Get-DscResource
foreach ($ds in $DscResources) {
    $moduleName = ''
    if ($ds.ModuleName) { $moduleName = $ds.ModuleName }
    elseif ($ds.ParentPath) { $moduleName = Split-Path $ds.ParentPath | Split-Path | Split-Path -Leaf }

    $resourceCache += [resourceCache]@{
        Type            = "$moduleName/$($ds.ResourceType)"
        DscResourceInfo = $ds
    }
}

# normalize the INPUT object to an array of configFormat objects
$inputObj = $stdIn | ConvertFrom-Json
$desiredState = @()
if ($inputObj.resources) {
    $context = 'config'
    # change the type from pscustomobject to configFormat
    $inputObj.resources | ForEach-Object -Process {
        $desiredState += [configFormat]@{
            name       = $_.name
            type       = $_.type
            properties = $_.properties
        }
    }
}
else {
    $context = 'resource'
    # mimic a config object with a single resource
    $type = $inputObj.type
    $inputObj.psobject.properties.Remove('type')
    $desiredState += [configFormat]@{
        name       = 'dsc resource'
        type       = $type
        properties = $inputObj
    }
}

function Get-Resource {
    param(
        [Alias('ds')]
        [Parameter(Mandatory)]
        [configFormat]$DesiredState,
        [Parameter(Mandatory)]
        [resourceCache[]]$ResourceCache
    )
    # get details from cache about the DSC resource, if it exists
    $cachedResourceInfo = $ResourceCache | Where-Object Type -EQ $ds.type | ForEach-Object DscResourceInfo

    if ($cachedResourceInfo) {
        # if the resource is found in the cache from Get-DscResource

        # since we need to workaround script based resources, save an unmodified INPUT object
        $savedInput = $ds.psobject.copy()

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        if ($cachedResourceInfo.ImplementationDetail -EQ 'ScriptBased') {
            # imports the .psm1 file for the DSC resource as a PowerShell module and stores the list of parameters
            Import-Module -Scope Local -Name $cachedResourceInfo.path -Force -ErrorAction stop
            $validParams = (Get-Command -Module $cachedResourceInfo.ResourceType -Name 'Get-TargetResource').Parameters.Keys
            # prune any properties that are not valid parameters of Get-TargetResource
            $ds.properties.psobject.properties | ForEach-Object -Process {
                if ($validParams -notcontains $_.Name) {
                    $ds.properties.psobject.properties.Remove($_.Name)
                }
            }
        }

        # morph the INPUT object into a hashtable
        $ds.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

        # using the cmdlet from PSDesiredStateConfiguration module, and handle errors
        try {
            $getResult = Invoke-DscResource -Method Get -ModuleName $cachedResourceInfo.ModuleName -Name $cachedResourceInfo.Name -Property $property
        }
        catch {
            Write-Error $_.Exception.Message
            exit 1
        }

        # formated OUTPUT of each resource
        $addToActualState = [configFormat]@{}

        # set top level properties of the OUTPUT object from saved INPUT object
        $savedInput.psobject.properties | ForEach-Object -Process {
            if ($_.TypeNameOfValue -EQ 'System.String') { $addToActualState.$($_.Name) = $savedInput.($_.Name) }
        }

        # set the properties of the OUTPUT object from the result of Get-TargetResource
        $addToActualState.properties = $getResult
        return $addToActualState
    }
    else {
        $errmsg = 'Can not find type ' + $ds.type + '; please ensure that Get-DscResource returns this resource type'
        Write-Error $errmsg
        exit 1
    }
}

if ($Operation -EQ 'List') {
    #TODO: following should be added to debug stream of every operation
    #$m = gmo PSDesiredStateConfiguration
    #$ds += @{"DebugInfo"=@{"ModuleVersion"=$m.Version.ToString();"ModulePath"=$m.Path;"PSVersion"=$PSVersionTable.PSVersion.ToString();"PSPath"=$PSHome}}
    #$ds[0] | ConvertTo-Json -Compress -Depth 3

    # Type is a property representing each known DSC Resource as modulename/resourcename
    foreach ($Type in $resourceCache.Type) {

        # https://learn.microsoft.com/dotnet/api/system.management.automation.dscresourceinfo
        $r = $resourceCache | Where-Object Type -EQ $Type | ForEach-Object DscResourceInfo

        # Binary resources are not supported in PowerShell 7
        if ($r.ImplementedAs -EQ 'Binary') {
            continue
        }

        if ($null -ne $r.moduleName) {
            if ($WinPS) { $requiresString = 'DSC/WindowsPowerShellGroup' } else { $requiresString = 'DSC/PowerShellGroup' }

            # dsc is expecting a json object with the following properties
            $resourceOutput = [resourceOutput]@{
                type          = $Type
                version       = $r.version.ToString()
                path          = $r.Path
                directory     = $r.ParentPath
                implementedAs = $r.ImplementationDetail
                author        = $r.CompanyName
                properties    = $r.Properties.Name
                requires      = $requiresString
            }
            $resourceOutput | ConvertTo-Json -Compress
        }
    }
}
elseif ($Operation -EQ 'Get') {
    # initialize OUTPUT as array
    $actualState = @()

    foreach ($ds in $desiredState) {
        # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
        $addToActualState = Get-Resource -DesiredState $ds -ResourceCache $resourceCache
        # add resource actual state to the OUTPUT object
        $actualState += $addToActualState
    }

    # OUTPUT
    @{resources = $actualState} | ConvertTo-Json -Depth 5
}
elseif ($Operation -EQ 'Set') {
    # in version 3, the output of Set includes information from Get

    # initialize OUTPUT as array
    $actualState = @()

    foreach ($ds in $desiredState) {
        # store the INPUT to each resource as resourceInfo and the OUTPUT as result

        # get details from cache about the DSC resource, if it exists
        $cachedResourceInfo = $resourceCache | Where-Object Type -EQ $ds.type | ForEach-Object DscResourceInfo

        if ($cachedResourceInfo) {
            # if the resource is found in the cache from Get-DscResource

            # morph the INPUT object into a hashtable
            $ds.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

            # using the cmdlet from PSDesiredStateConfiguration module, and handle errors
            try {
                $setResult = Invoke-DscResource -Method Set -ModuleName $cachedResourceInfo.ModuleName -Name $cachedResourceInfo.Name -Property $property
            }
            catch {
                Write-Error $_.Exception.Message
                exit 1
            }

            # set the properties of the OUTPUT object from the result of Get-TargetResource
            $addToActualState = Get-Resource -DesiredState $ds -ResourceCache $resourceCache

            # add the properties of the OUTPUT object from the result of Set-TargetResource
            $setResult.psobject.Properties | ForEach-Object -Process {
                $addToActualState.properties | Add-Member -Type NoteProperty -Name $_.Name -Value $_.Value
            }

            # add resource actual state to the OUTPUT object
            $actualState += $addToActualState
        }
        else {
            $errmsg = 'Can not find type ' + $ds.type + '; please ensure that Get-DscResource returns this resource type'
            Write-Error $errmsg
            exit 1
        }
    }

    # OUTPUT
    @{resources = $actualState} | ConvertTo-Json -Depth 5
}
elseif ($Operation -EQ 'Test') {
    # in version 3, the output of Test includes information from Get

    # initialize OUTPUT as array
    $actualState = @()

    foreach ($ds in $desiredState) {
        # store the INPUT to each resource as resourceInfo and the OUTPUT as result

        # get details from cache about the DSC resource, if it exists
        $cachedResourceInfo = $resourceCache | Where-Object Type -EQ $ds.type | ForEach-Object DscResourceInfo

        if ($cachedResourceInfo) {
            # if the resource is found in the cache from Get-DscResource

            # morph the INPUT object into a hashtable
            $ds.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

            # using the cmdlet from PSDesiredStateConfiguration module, and handle errors
            try {
                $testResult = Invoke-DscResource -Method Test -ModuleName $cachedResourceInfo.ModuleName -Name $cachedResourceInfo.Name -Property $property
            }
            catch {
                Write-Error $_.Exception.Message
                exit 1
            }

            # set the properties of the OUTPUT object from the result of Get-TargetResource
            $addToActualState = Get-Resource -DesiredState $ds -ResourceCache $resourceCache

            # add the properties of the OUTPUT object from the result of Set-TargetResource
            $testResult.psobject.Properties | ForEach-Object -Process {
                $addToActualState.properties | Add-Member -Type NoteProperty -Name $_.Name -Value $_.Value
            }

            # add resource actual state to the OUTPUT object
            $actualState += $addToActualState
        }
        else {
            # if the resource is not found in the cache from Get-DscResource
            $errmsg = 'Can not find type ' + $inputObj.type + '; please ensure that Get-DscResource returns this resource type'
            Write-Error $errmsg
            exit 1
        }
    }

    # OUTPUT
    @{resources = $actualState} | ConvertTo-Json -Depth 5
}
else {
    'ERROR: Unsupported operation requested from powershellgroup.resource.ps1'
}

# cached resource standard
class resourceCache {
    [string] $Type
    [PSObject] $DscResourceInfo
}

# format expected for config input and output
class configFormat {
    [string] $name
    [string] $type
    [PSObject] $properties
}

# output standard
class resourceOutput {
    [string] $type
    [string] $version
    [string] $path
    [string] $directory
    [string] $implementedAs
    [string] $author
    [string[]] $properties
    [string] $requires
}