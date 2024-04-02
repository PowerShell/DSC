# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '@{}'
)

# load private functions of psDscAdapter stub module
Import-Module '$PSScriptRoot/psDscAdapter/psDscAdapter.psd1' -Force

# cached resource
class resourceCache {
    [string] $Type
    [psobject] $DscResourceInfo
}

# format expected for configuration and resource output
class configFormat {
    [string] $name
    [string] $type
    [psobject[]] $properties
}

# output format for resource list
class resourceOutput {
    [string] $type
    [string] $kind
    [string] $version
    [string[]] $capabilities
    [string] $path
    [string] $directory
    [string] $implementedAs
    [string] $author
    [string[]] $properties
    [string] $requireAdapter
    [string] $description
}

# module types
enum moduleType {
    ScriptBased
    ClassBased
}

# dsc resource type (settable clone)
class DscResourceInfo {
    [moduleType] $ImplementationDetail
    [string] $ResourceType
    [string] $Name
    [string] $FriendlyName
    [string] $Module
    [string] $ModuleName
    [string] $Version
    [string] $Path
    [string] $ParentPath
    [string] $ImplementedAs
    [string] $CompanyName
    [psobject[]] $Properties
}

# Cache the results of Get-DscResource to optimize performance
function Invoke-CacheRefresh {
    param(
        [Parameter(Mandatory = $false)]
        [string[]] $module
    )
    # cache the results of Get-DscResource
    [resourceCache[]]$resourceCache = @()

    # improve by performance by having the option to only get details for named modules
    if ($null -ne $module) {
        if ($module.gettype().name -eq 'string') {
            $module = @($module)
        }
        $DscResources = @()
        $Modules = @()
        foreach ($m in $module) {
            $DscResources += psDscAdapter\Get-DscResource -Module $m
            $Modules += Get-Module -Name $m -ListAvailable
        }
    }
    else {
        $DscResources = psDscAdapter\Get-DscResource
        $Modules = Get-Module -ListAvailable
    }

    foreach ($dsc in $DscResources) {
        # only support known moduleType, excluding binary
        if ([moduleType].GetEnumNames() -notcontains $dsc.ImplementationDetail) {
            continue
        }
        # workaround: if the resource does not have a module name, get it from parent path
        # workaround: modulename is not settable, so clone the object without being read-only
        $DscResourceInfo = [DscResourceInfo]::new()
        $dsc.PSObject.Properties | ForEach-Object -Process { $DscResourceInfo.$($_.Name) = $_.Value }
        if ($dsc.ModuleName) {
            $moduleName = $dsc.ModuleName
        }
        elseif ($dsc.ParentPath) {
            # workaround: populate module name from parent path that is three levels up
            $moduleName = Split-Path $dsc.ParentPath | Split-Path | Split-Path -Leaf
            $DscResourceInfo.Module = $moduleName
            $DscResourceInfo.ModuleName = $moduleName
            # workaround: populate module version from psmoduleinfo if available
            if ($moduleInfo = $Modules | Where-Object { $_.Name -eq $moduleName }) {
                $moduleInfo = $moduleInfo | Sort-Object -Property Version -Descending | Select-Object -First 1
                $DscResourceInfo.Version = $moduleInfo.Version.ToString()
            }
        }

        $resourceCache += [resourceCache]@{
            Type            = "$moduleName/$($dsc.Name)"
            DscResourceInfo = $DscResourceInfo
        }
    }
    return $resourceCache
}

# Convert the INPUT to a configFormat object so configuration and resource are standardized as moch as possible
function Get-ConfigObject {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        $jsonInput
    )
    # normalize the INPUT object to an array of configFormat objects
    $inputObj = $jsonInput | ConvertFrom-Json
    $desiredState = @()

    # catch potential for improperly formatted configuration input
    if ($inputObj.resources -and -not $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        Write-Warning 'The input has a top level property named "resources" but is not a configuration. If the input should be a configuration, include the property: "metadata": {"Microsoft.DSC": {"context": "Configuration"}}'
    }

    if ($null -ne $inputObj.metadata -and $null -ne $inputObj.metadata.'Microsoft.DSC' -and $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
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
        # mimic a config object with a single resource
        $type = $inputObj.type
        $inputObj.psobject.properties.Remove('type')
        $desiredState += [configFormat]@{
            name       = 'Microsoft.Dsc/PowerShell'
            type       = $type
            properties = $inputObj
        }
    }
    return $desiredState
}

# Get-ActualState function to get the actual state of the resource
function Get-ActualState {
    param(
        [Parameter(Mandatory, ValueFromPipeline = $true)]
        [configFormat]$DesiredState,
        [Parameter(Mandatory)]
        [resourceCache[]]$ResourceCache
    )
    # get details from cache about the DSC resource, if it exists
    $cachedResourceInfo = $ResourceCache | Where-Object Type -EQ $DesiredState.type | ForEach-Object DscResourceInfo

    # if the resource is found in the cache, get the actual state
    if ($cachedResourceInfo) {

        # formated OUTPUT of each resource
        $addToActualState = [configFormat]@{}

        # set top level properties of the OUTPUT object from INPUT object
        $DesiredState.psobject.properties | ForEach-Object -Process {
            if ($_.TypeNameOfValue -EQ 'System.String') { $addToActualState.$($_.Name) = $DesiredState.($_.Name) }
        }

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        switch ([moduleType]$cachedResourceInfo.ImplementationDetail) {
            'ScriptBased' {

                # If the OS is Windows, import the embedded psDscAdapter module. For Linux/MacOS, only class based resources are supported and are called directly.
                if (!$IsWindows) {
                    Write-Error 'Script based resources are only supported on Windows.'
                    exit 1
                }

                # imports the .psm1 file for the DSC resource as a PowerShell module and stores the list of parameters
                Import-Module -Scope Local -Name $cachedResourceInfo.path -Force -ErrorAction stop
                $validParams = (Get-Command -Module $cachedResourceInfo.ResourceType -Name 'Get-TargetResource').Parameters.Keys
                # prune any properties that are not valid parameters of Get-TargetResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                    if ($validParams -notcontains $_.Name) {
                        $DesiredState.properties.psobject.properties.Remove($_.Name)
                    }
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

                # using the cmdlet from psDscAdapter module, and handle errors
                try {
                    $getResult = psDscAdapter\Invoke-DscResource -Method Get -ModuleName $cachedResourceInfo.ModuleName -Name $cachedResourceInfo.Name -Property $property

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getResult
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            }
            'ClassBased' {
                try {
                    # load powershell class from external module
                    $resource = Get-TypeInstanceFromModule -modulename $cachedResourceInfo.ModuleName -classname $cachedResourceInfo.Name
                    $resourceInstance = $resource::New()

                    # set each property of $resourceInstance to the value of the property in the $desiredState INPUT object
                    $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                        $resourceInstance.$($_.Name) = $_.Value
                    }
                    $getResult = $resourceInstance.Get()

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getResult
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            }
            Default {
                $errmsg = 'Can not find implementation of type: "' + $cachedResourceInfo.ImplementationDetail + '". If this is a binary resource such as File, use the Microsoft.Dsc/WindowsPowerShell adapter.'
                Write-Error $errmsg
                exit 1
            }
        }

        return $addToActualState
    }
    else {
        $dsJSON = $DesiredState | ConvertTo-Json -Depth 10
        $errmsg = 'Can not find type "' + $DesiredState.type + '" for resource "' + $dsJSON + '". Please ensure that Get-DscResource returns this resource type.'
        Write-Error $errmsg
        exit 1
    }
}

# Get-TypeInstanceFromModule function to get the type instance from the module
function Get-TypeInstanceFromModule {
    param(
        [Parameter(Mandatory = $true)]
        [string] $modulename,
        [Parameter(Mandatory = $true)]
        [string] $classname
    )
    $instance = & (Import-Module $modulename -PassThru) ([scriptblock]::Create("'$classname' -as 'type'"))
    return $instance
}

# initialize OUTPUT as array
$result = [System.Collections.Generic.List[Object]]::new()

# process the operation requested to the script
switch ($Operation) {
    'List' {
        $resourceCache = Invoke-CacheRefresh

        # cache was refreshed on script load
        foreach ($Type in $resourceCache.Type) {
        
            # https://learn.microsoft.com/dotnet/api/system.management.automation.dscresourceinfo
            $r = $resourceCache | Where-Object Type -EQ $Type | ForEach-Object DscResourceInfo

            # Provide a way for existing resources to specify their capabilities, or default to Get, Set, Test
            $module = Get-Module -Name $r.ModuleName -ListAvailable | Sort-Object -Property Version -Descending | Select-Object -First 1
            if ($module.PrivateData.PSData.DscCapabilities) {
                $capabilities = $module.PrivateData.PSData.DscCapabilities
            }
            else {
                $capabilities = @('Get', 'Set', 'Test')
            }

            # this text comes directly from the resource manifest for v3 native resources
            if ($r.Description) {
                $description = $r.Description
            }
            else {
                # some modules have long multi-line descriptions. to avoid issue, use only the first line.
                $description = $module.Description.split("`r`n")[0]
            }

            # OUTPUT dsc is expecting the following properties
            [resourceOutput]@{
                type           = $Type
                kind           = 'Resource'
                version        = $r.version.ToString()
                capabilities   = $capabilities
                path           = $r.Path
                directory      = $r.ParentPath
                implementedAs  = $r.ImplementationDetail
                author         = $r.CompanyName
                properties     = $r.Properties.Name
                requireAdapter = 'Microsoft.Dsc/PowerShell'
                description    = $description
            } | ConvertTo-Json -Compress
        }
    }
    'Get' {
        $desiredState = $jsonInput | Get-ConfigObject

        # only need to cache the resources that are used
        $resourceCache = Invoke-CacheRefresh -module ($desiredState | ForEach-Object {$_.Type.Split('/')[0]})

        foreach ($ds in $desiredState) {
            # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
            $result += Get-ActualState -DesiredState $ds -ResourceCache $resourceCache
        }
    
        # OUTPUT
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Set' {
        throw 'SET not implemented'
        
        # OUTPUT
        $result += @{}
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Test' {
        throw 'TEST not implemented'
        
        # OUTPUT
        $result += @{}
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Export' {
        throw 'EXPORT not implemented'
        
        # OUTPUT
        $result += @{}
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Validate' {
        # VALIDATE not implemented
        
        # OUTPUT
        @{ valid = $true } | ConvertTo-Json
    }
    Default {
        Write-Error 'Unsupported operation. Please use one of the following: List, Get, Set, Test, Export, Validate'
    }
}

# Adding some debug info to STDERR
$trace = @{'Debug' = 'PSVersion=' + $PSVersionTable.PSVersion.ToString() } | ConvertTo-Json -Compress
$host.ui.WriteErrorLine($trace)
$trace = @{'Debug' = 'PSPath=' + $PSHome } | ConvertTo-Json -Compress
$host.ui.WriteErrorLine($trace)
$m = Get-Command 'Get-DscResource'
$trace = @{'Debug' = 'Module=' + $m.Source.ToString() } | ConvertTo-Json -Compress
$host.ui.WriteErrorLine($trace)
$trace = @{'Debug' = 'PSModulePath=' + $env:PSModulePath } | ConvertTo-Json -Compress
$host.ui.WriteErrorLine($trace)