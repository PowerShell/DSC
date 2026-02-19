# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$script:CurrentCacheSchemaVersion = 3
$ErrorActionPreference = 'Stop'

function Import-PSDSCModule {
    $m = Get-Module PSDesiredStateConfiguration -ListAvailable | Sort-Object -Descending | Select-Object -First 1
    $PSDesiredStateConfiguration = Import-Module $m -Force -PassThru
}

function Get-DSCResourceModules {
    $listPSModuleFolders = $env:PSModulePath.Split([IO.Path]::PathSeparator)
    $dscModulePsd1List = [System.Collections.Generic.HashSet[System.String]]::new()
    foreach ($folder in $listPSModuleFolders) {
        if (!(Test-Path -LiteralPath $folder -ErrorAction Ignore)) {
            continue
        }

        foreach ($moduleFolder in Get-ChildItem -LiteralPath $folder -Directory -ErrorAction Ignore) {
            foreach ($psd1 in Get-ChildItem -Recurse -Filter "$($moduleFolder.Name).psd1" -LiteralPath $moduleFolder.fullname -Depth 2 -ErrorAction Ignore) {
                $containsDSCResource = select-string -LiteralPath $psd1 -pattern '^[^#]*\bDscResourcesToExport\b.*'
                if ($null -ne $containsDSCResource) {
                    $dscModulePsd1List.Add($psd1) | Out-Null
                }
            }
        }
    }

    return $dscModulePsd1List
}

function Add-AstMembers {
    param(
        $AllTypeDefinitions,
        $TypeAst,
        $Properties
    )

    foreach ($TypeConstraint in $TypeAst.BaseTypes) {
        $t = $AllTypeDefinitions | Where-Object { $_.Name -eq $TypeConstraint.TypeName.Name }
        if ($t) {
            Add-AstMembers $AllTypeDefinitions $t $Properties
        }
    }

    foreach ($member in $TypeAst.Members) {
        $property = $member -as [System.Management.Automation.Language.PropertyMemberAst]
        if (($null -eq $property) -or ($property.IsStatic)) {
            continue;
        }
        $skipProperty = $true
        $isKeyProperty = $false
        foreach ($attr in $property.Attributes) {
            if ($attr.TypeName.Name -eq 'DscProperty') {
                $skipProperty = $false
                foreach ($attrArg in $attr.NamedArguments) {
                    if ($attrArg.ArgumentName -eq 'Key') {
                        $isKeyProperty = $true
                        break
                    }
                }
            }
        }
        if ($skipProperty) {
            continue;
        }

        [DscResourcePropertyInfo]$prop = [DscResourcePropertyInfo]::new()
        $prop.Name = $property.Name
        $prop.PropertyType = $property.PropertyType.TypeName.Name
        $prop.IsMandatory = $isKeyProperty
        $Properties.Add($prop)
    }
}

function FindAndParseResourceDefinitions {
    [CmdletBinding(HelpUri = '')]
    param(
        [Parameter(Mandatory = $true)]
        [string]$filePath,
        [Parameter(Mandatory = $true)]
        [string]$moduleVersion
    )

    if (-not (Test-Path $filePath)) {
        return
    }

    if (".psm1", ".ps1" -notcontains ([System.IO.Path]::GetExtension($filePath))) {
        return
    }

    Write-Debug -Debug ("Loading resources from file '$filePath'")
    #TODO: Ensure embedded instances in properties are working correctly
    [System.Management.Automation.Language.Token[]] $tokens = $null
    [System.Management.Automation.Language.ParseError[]] $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile($filePath, [ref]$tokens, [ref]$errors)
    foreach ($e in $errors) {
        $e | Out-String | Write-Error
    }

    $typeDefinitions = $ast.FindAll(
        {
            $typeAst = $args[0] -as [System.Management.Automation.Language.TypeDefinitionAst]
            return $null -ne $typeAst;
        },
        $false);

    $resourceList = [System.Collections.Generic.List[DscResourceInfo]]::new()

    foreach ($typeDefinitionAst in $typeDefinitions) {
        foreach ($a in $typeDefinitionAst.Attributes) {
            if ($a.TypeName.Name -eq 'DscResource') {
                $DscResourceInfo = [DscResourceInfo]::new()
                $DscResourceInfo.Name = $typeDefinitionAst.Name
                $DscResourceInfo.ResourceType = $typeDefinitionAst.Name
                $DscResourceInfo.FriendlyName = $typeDefinitionAst.Name
                $DscResourceInfo.ImplementationDetail = 'ClassBased'
                $DscResourceInfo.Module = $filePath
                $DscResourceInfo.Path = $filePath
                #TODO: ModuleName, Version and ParentPath should be taken from psd1 contents
                $DscResourceInfo.ModuleName = [System.IO.Path]::GetFileNameWithoutExtension($filePath)
                $DscResourceInfo.ParentPath = [System.IO.Path]::GetDirectoryName($filePath)
                $DscResourceInfo.Version = $moduleVersion

                $DscResourceInfo.Properties = [System.Collections.Generic.List[DscResourcePropertyInfo]]::new()
                $DscResourceInfo.Capabilities = GetClassBasedCapabilities $typeDefinitionAst.Members
                Add-AstMembers $typeDefinitions $typeDefinitionAst $DscResourceInfo.Properties

                $resourceList.Add($DscResourceInfo)
            }
        }
    }

    return $resourceList
}

function GetExportMethod ($ResourceType, $HasFilterProperties, $ResourceTypeName) {
    $methods = $ResourceType.GetMethods() | Where-Object { $_.Name -eq 'Export' }
    $method = $null

    if ($HasFilterProperties) {
        Write-Verbose -Verbose "Properties provided for filtered export"
        $method = foreach ($mt in $methods) {
            if ($mt.GetParameters().Count -gt 0) {
                $mt
                break
            }
        }

        if ($null -eq $method) {
            Write-Error ("Export method with parameters not implemented by resource '$ResourceTypeName'. Filtered export is not supported.")
            exit 1
        }
    }
    else {
        Write-Verbose -Verbose "No properties provided, using parameterless export"
        $method = foreach ($mt in $methods) {
            if ($mt.GetParameters().Count -eq 0) {
                $mt
                break
            }
        }

        if ($null -eq $method) {
            Write-Error ("Export method not implemented by resource '$ResourceTypeName'")
            exit 1
        }
    }

    return $method
}

function LoadPowerShellClassResourcesFromModule {
    [CmdletBinding(HelpUri = '')]
    param(
        [Parameter(Mandatory = $true)]
        [PSModuleInfo]$moduleInfo
    )

    Write-Debug -Debug ("Loading resources from module '$($moduleInfo.Path)'")

    if ($moduleInfo.RootModule) {
        if (".psm1", ".ps1" -notcontains ([System.IO.Path]::GetExtension($moduleInfo.RootModule)) -and
            (-not $moduleInfo.NestedModules)) {
            Write-Debug -Debug ("RootModule is neither psm1 nor ps1 '$($moduleInfo.RootModule)'")
            return
        }

        $scriptPath = Join-Path $moduleInfo.ModuleBase  $moduleInfo.RootModule
    }
    else {
        $scriptPath = $moduleInfo.Path;
    }

    $version = if ($moduleInfo.Version) { $moduleInfo.Version.ToString() } else { '0.0.0' }
    $Resources = FindAndParseResourceDefinitions $scriptPath $version

    if ($moduleInfo.NestedModules) {
        foreach ($nestedModule in $moduleInfo.NestedModules) {
            $resourcesOfNestedModules = LoadPowerShellClassResourcesFromModule $nestedModule
            if ($resourcesOfNestedModules) {
                $Resources.AddRange($resourcesOfNestedModules)
            }
        }
    }

    return $Resources
}

<# public function Invoke-DscCacheRefresh
.SYNOPSIS
    This function caches the results of the Get-DscResource call to optimize performance.

.DESCRIPTION
    This function is designed to improve the performance of DSC operations by caching the results of the Get-DscResource call.
    By storing the results, subsequent calls to Get-DscResource can retrieve the cached data instead of making a new call each time.
    This can significantly speed up operations that need to repeatedly access DSC resources.

.EXAMPLE
    Invoke-DscCacheRefresh -Module "PSDesiredStateConfiguration"
#>
function Invoke-DscCacheRefresh {
    [CmdletBinding(HelpUri = '')]
    param(
        [Parameter(ValueFromPipeline = $true, ValueFromPipelineByPropertyName = $true)]
        [Object[]]
        $Module
    )

    $refreshCache = $false

    $cacheFilePath = if ($IsWindows) {
        # PS 6+ on Windows
        Join-Path $env:LocalAppData "dsc\PSAdapterCache.json"
    }
    else {
        # PS 6+ on Linux/Mac
        Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
    }

    if (Test-Path $cacheFilePath) {
        Write-Verbose -Verbose ("Reading from Get-DscResource cache file $cacheFilePath")

        $cache = Get-Content -Raw $cacheFilePath | ConvertFrom-Json

        if ($cache.CacheSchemaVersion -ne $script:CurrentCacheSchemaVersion) {
            $refreshCache = $true
            Write-Verbose -Verbose ("Incompatible version of cache in file '" + $cache.CacheSchemaVersion + "' (expected '" + $script:CurrentCacheSchemaVersion + "')")
        }
        else {
            $dscResourceCacheEntries = $cache.ResourceCache

            if ($dscResourceCacheEntries.Count -eq 0) {
                # if there is nothing in the cache file - refresh cache
                $refreshCache = $true

                Write-Debug -Debug "Filtered DscResourceCache cache is empty"
            }
            else {
                Write-Debug -Debug "Checking cache for stale entries"

                foreach ($cacheEntry in $dscResourceCacheEntries) {

                    $cacheEntry.LastWriteTimes.PSObject.Properties | ForEach-Object {

                        if (Test-Path $_.Name) {
                            $file_LastWriteTime = (Get-Item $_.Name).LastWriteTime
                            # Truncate DateTime to seconds
                            $file_LastWriteTime = $file_LastWriteTime.AddTicks( - ($file_LastWriteTime.Ticks % [TimeSpan]::TicksPerSecond));

                            $cache_LastWriteTime = [DateTime]$_.Value
                            # Truncate DateTime to seconds
                            $cache_LastWriteTime = $cache_LastWriteTime.AddTicks( - ($cache_LastWriteTime.Ticks % [TimeSpan]::TicksPerSecond));

                            if (-not ($file_LastWriteTime.Equals($cache_LastWriteTime))) {
                                Write-Debug -Debug "Detected stale cache entry '$($_.Name)'"
                                $refreshCache = $true
                                break
                            }
                        }
                        else {
                            Write-Debug -Debug ("Detected non-existent cache entry '$($_.Name)'")
                            $refreshCache = $true
                            break
                        }
                    }

                    if ($refreshCache) { break }
                }

                if (-not $refreshCache) {
                    Write-Debug -Debug "Checking cache for stale PSModulePath"

                    $m = $env:PSModulePath -split [IO.Path]::PathSeparator | % { Get-ChildItem -Directory -LiteralPath $_ -Depth 1 -ErrorAction Ignore }

                    $hs_cache = [System.Collections.Generic.HashSet[string]]($cache.PSModulePaths)
                    $hs_live = [System.Collections.Generic.HashSet[string]]($m.FullName)
                    $hs_cache.SymmetricExceptWith($hs_live)
                    $diff = $hs_cache

                    Write-Debug -Debug ("PSModulePath diff '$diff'")
                    if ($diff.Count -gt 0) {
                        $refreshCache = $true
                    }
                }
            }
        }
    }
    else {
        Write-Verbose -Verbose ("Cache file not found '$cacheFilePath'")
        $refreshCache = $true
    }

    if ($refreshCache) {
        Write-Verbose -Verbose "Constructing Get-DscResource cache"

        # create a list object to store cache of Get-DscResource
        [dscResourceCacheEntry[]]$dscResourceCacheEntries = [System.Collections.Generic.List[Object]]::new()

        $DscResources = [System.Collections.Generic.List[DscResourceInfo]]::new()
        $dscResourceModulePsd1s = Get-DSCResourceModules
        if ($null -ne $dscResourceModulePsd1s) {
            $modules = Get-Module -ListAvailable -Name ($dscResourceModulePsd1s) -ErrorAction Ignore
            $processedModuleNames = @{}
            foreach ($mod in $modules) {
                if (-not ($processedModuleNames.ContainsKey($mod.Name))) {
                    $processedModuleNames.Add($mod.Name, $true)

                    # from several modules with the same name select the one with the highest version
                    $selectedMod = $modules | Where-Object Name -EQ $mod.Name
                    if ($selectedMod.Count -gt 1) {
                        Write-Debug -Debug ("Found $($selectedMod.Count) modules with name '$($mod.Name)'")
                        $selectedMod = $selectedMod | Sort-Object -Property Version -Descending | Select-Object -First 1
                    }

                    [System.Collections.Generic.List[DscResourceInfo]]$r = LoadPowerShellClassResourcesFromModule -moduleInfo $selectedMod
                    if ($r) {
                        $DscResources.AddRange($r)
                    }
                }
            }
        }

        foreach ($dscResource in $DscResources) {
            $moduleName = $dscResource.ModuleName

            # fill in resource files (and their last-write-times) that will be used for up-do-date checks
            $lastWriteTimes = @{}
            Get-ChildItem -Recurse -File -Path $dscResource.ParentPath -Include "*.ps1", "*.psd1", "*.psm1", "*.mof" -ErrorAction Ignore | ForEach-Object {
                $lastWriteTimes.Add($_.FullName, $_.LastWriteTime)
            }

            $dscResourceCacheEntries += [dscResourceCacheEntry]@{
                Type            = "$moduleName/$($dscResource.Name)"
                DscResourceInfo = $dscResource
                LastWriteTimes  = $lastWriteTimes
            }
        }

        [dscResourceCache]$cache = [dscResourceCache]::new()
        $cache.ResourceCache = $dscResourceCacheEntries
        $m = $env:PSModulePath -split [IO.Path]::PathSeparator | ForEach-Object { Get-ChildItem -Directory -Path $_ -Depth 1 -ErrorAction Ignore }
        $cache.PSModulePaths = $m.FullName
        $cache.CacheSchemaVersion = $script:CurrentCacheSchemaVersion

        # save cache for future use
        # TODO: replace this with a high-performance serializer
        Write-Debug -Debug ("Saving Get-DscResource cache to '$cacheFilePath'")
        $jsonCache = $cache | ConvertTo-Json -Depth 90
        New-Item -Force -Path $cacheFilePath -Value $jsonCache -Type File | Out-Null
    }

    return $dscResourceCacheEntries
}

# Convert the INPUT to a dscResourceObject object so configuration and resource are standardized as much as possible
function Get-DscResourceObject {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        $jsonInput,
        [Parameter(Mandatory = $false)]
        $type
    )
    # normalize the INPUT object to an array of dscResourceObject objects
    $inputObj = $jsonInput | ConvertFrom-Json
    if ($type) {
        $desiredState = [dscResourceObject]@{
            name       = ''
            type       = $type
            properties = $inputObj
        }
    }
    else {
        $desiredState = [System.Collections.Generic.List[Object]]::new()

        $inputObj.resources | ForEach-Object -Process {
            $desiredState += [dscResourceObject]@{
                name       = $_.name
                type       = $_.type
                properties = $_.properties
            }
        }
    }

    return $desiredState
}

# Get the actual state using DSC Get method from any type of DSC resource
function Invoke-DscOperation {
    param(
        [Parameter(Mandatory)]
        [ValidateSet('Get', 'Set', 'Test', 'Export')]
        [string]$Operation,
        [Parameter(Mandatory, ValueFromPipeline = $true)]
        [dscResourceObject]$DesiredState,
        [Parameter(Mandatory)]
        [dscResourceCacheEntry[]]$dscResourceCache
    )

    $osVersion = [System.Environment]::OSVersion.VersionString
    Write-Debug -Debug ('OS version: ' + $osVersion)

    $psVersion = $PSVersionTable.PSVersion.ToString()
    Write-Debug -Debug ('PowerShell version: ' + $psVersion)

    # get details from cache about the DSC resource, if it exists
    $cachedDscResourceInfo = $dscResourceCache | Where-Object Type -EQ $DesiredState.type | ForEach-Object DscResourceInfo | Select-Object -First 1

    # if the resource is found in the cache, get the actual state
    if ($cachedDscResourceInfo) {

        # formated OUTPUT of each resource
        $addToActualState = [dscResourceObject]@{}

        # set top level properties of the OUTPUT object from INPUT object
        $DesiredState.psobject.properties | ForEach-Object -Process {
            if ($_.TypeNameOfValue -EQ 'System.String') { $addToActualState.$($_.Name) = $DesiredState.($_.Name) }
        }

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        switch ([dscResourceType]$cachedDscResourceInfo.ImplementationDetail) {

            'ClassBased' {
                try {
                    # load powershell class from external module
                    $resource = GetTypeInstanceFromModule -modulename $cachedDscResourceInfo.ModuleName -classname $cachedDscResourceInfo.Name
                    $dscResourceInstance = $resource::New()

                    $ValidProperties = $cachedDscResourceInfo.Properties.Name

                    Write-Debug -Debug ("Valid properties: " + ($ValidProperties | ConvertTo-Json))

                    if ($DesiredState.properties) {
                        # set each property of $dscResourceInstance to the value of the property in the $desiredState INPUT object
                        $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                            # handle input objects by converting them to a hash table
                            $validateProperty = $cachedDscResourceInfo.Properties | Where-Object -Property Name -EQ $_.Name
                            if ($_.Value -is [System.Management.Automation.PSCustomObject]) {
                                if ($validateProperty -and $validateProperty.PropertyType -in @('PSCredential', 'System.Management.Automation.PSCredential')) {
                                    if (-not $_.Value.Username -or -not $_.Value.Password) {
                                        Write-Error ("Credential object '$($_.Name)' requires both 'username' and 'password' properties")
                                        exit 1
                                    }
                                    $dscResourceInstance.$($_.Name) = [System.Management.Automation.PSCredential]::new($_.Value.Username, (ConvertTo-SecureString -AsPlainText $_.Value.Password -Force))
                                }
                                else {
                                    $dscResourceInstance.$($_.Name) = $_.Value.psobject.properties | ForEach-Object -Begin { $propertyHash = @{} } -Process { $propertyHash[$_.Name] = $_.Value } -End { $propertyHash }
                                }
                            }
                            else {
                                if ($validateProperty -and $validateProperty.PropertyType -in @('SecureString', 'System.Security.SecureString') -and -not [string]::IsNullOrEmpty($_.Value)) {
                                    $dscResourceInstance.$($_.Name) = ConvertTo-SecureString -AsPlainText $_.Value -Force
                                } else {
                                    $dscResourceInstance.$($_.Name) = $_.Value
                                }
                            }
                        }
                    }

                    switch ($Operation) {
                        'Get' {
                            $Result = @{}
                            $raw_obj = $dscResourceInstance.Get()
                            $ValidProperties | ForEach-Object {
                                if ($raw_obj.$_ -is [System.Enum]) {
                                    $Result[$_] = $raw_obj.$_.ToString()

                                }
                                else {
                                    $Result[$_] = $raw_obj.$_
                                }
                            }
                            $addToActualState.properties = $Result
                        }
                        'Set' {
                            $dscResourceInstance.Set()
                        }
                        'Test' {
                            $Result = $dscResourceInstance.Test()
                            $addToActualState.properties = [psobject]@{'InDesiredState' = $Result }
                        }
                        'Export' {
                            $t = $dscResourceInstance.GetType()
                            $hasFilter = $null -ne $DesiredState.properties -and
                            ($DesiredState.properties.PSObject.Properties | Measure-Object).Count -gt 0

                            $method = GetExportMethod -ResourceType $t -HasFilterProperties $hasFilter -ResourceTypeName $DesiredState.Type

                            $resultArray = @()
                            if ($hasFilter) {
                                $raw_obj_array = $method.Invoke($null, @($dscResourceInstance))
                            } else {
                                $raw_obj_array = $method.Invoke($null, $null)
                            }

                            foreach ($raw_obj in $raw_obj_array) {
                                $Result_obj = @{}
                                $ValidProperties | ForEach-Object {
                                    if ($raw_obj.$_ -is [System.Enum]) {
                                        $Result_obj[$_] = $raw_obj.$_.ToString()
                                    }
                                    else {
                                        $Result_obj[$_] = $raw_obj.$_
                                    }
                                }
                                $resultArray += $Result_obj
                            }
                            $addToActualState = $resultArray
                        }
                    }
                }
                catch {

                    Write-Error ('Exception: ' + $_.Exception.Message)
                    exit 1
                }
            }
            Default {
                Write-Error ('Resource ImplementationDetail not supported: ' + $cachedDscResourceInfo.ImplementationDetail)
                exit 1
            }
        }

        Write-Debug -Debug ("Output: $($addToActualState | ConvertTo-Json -Depth 10 -Compress)")
        return $addToActualState
    }
    else {
        $dsJSON = $DesiredState | ConvertTo-Json -Depth 10
        Write-Error ('Can not find type "' + $DesiredState.type + '" for resource "' + $dsJSON + '". Please ensure that Get-DscResource returns this resource type.')
        exit 1
    }
}

# GetTypeInstanceFromModule function to get the type instance from the module
function GetTypeInstanceFromModule {
    param(
        [Parameter(Mandatory = $true)]
        [string] $modulename,
        [Parameter(Mandatory = $true)]
        [string] $classname
    )
    $instance = & (Import-Module $modulename -PassThru) ([scriptblock]::Create("'$classname' -as 'type'"))
    return $instance
}

function GetClassBasedCapabilities ($functionMemberAst) {
    $capabilities = [System.Collections.Generic.List[string[]]]::new()
    # These are the methods that we can potentially expect in a class-based DSC resource.
    $availableMethods = @('get', 'set', 'setHandlesExist', 'whatIf', 'test', 'delete', 'export')
    $methods = $functionMemberAst | Where-Object { $_ -is [System.Management.Automation.Language.FunctionMemberAst] -and $_.Name -in $availableMethods }

    foreach ($method in $methods.Name) {
        # We go through each method to properly case handle the method names.
        switch ($method) {
            'Get' { $capabilities.Add('get') }
            'Set' { $capabilities.Add('set') }
            'Test' { $capabilities.Add('test') }
            'WhatIf' { $capabilities.Add('whatIf') }
            'SetHandlesExist' { $capabilities.Add('setHandlesExist') }
            'Delete' { $capabilities.Add('delete') }
            'Export' { $capabilities.Add('export') }
        }
    }

    return ($capabilities | Select-Object -Unique)
}

# cached resource
class dscResourceCacheEntry {
    [string] $Type
    [psobject] $DscResourceInfo
    [PSCustomObject] $LastWriteTimes
}

class dscResourceCache {
    [int] $CacheSchemaVersion
    [string[]] $PSModulePaths
    [dscResourceCacheEntry[]] $ResourceCache
}

# format expected for configuration output
class dscResourceObject {
    [string] $name
    [string] $type
    [psobject] $properties
}

# dsc resource types
enum dscResourceType {
    ScriptBased
    ClassBased
    Binary
    Composite
}

class DscResourcePropertyInfo {
    [string] $Name
    [string] $PropertyType
    [bool] $IsMandatory
    [System.Collections.Generic.List[string]] $Values
}

# dsc resource type (settable clone)
class DscResourceInfo {
    [dscResourceType] $ImplementationDetail
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
    [System.Collections.Generic.List[DscResourcePropertyInfo]] $Properties
    [string[]] $Capabilities
}
