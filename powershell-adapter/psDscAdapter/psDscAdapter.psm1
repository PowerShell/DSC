# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$script:CurrentCacheSchemaVersion = 2

function Write-DscTrace {
    param(
        [Parameter(Mandatory = $false)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Operation = 'Debug',

        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{$Operation = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

function Import-PSDSCModule {
    $m = Get-Module PSDesiredStateConfiguration -ListAvailable | Sort-Object -Descending | Select-Object -First 1
    $PSDesiredStateConfiguration = Import-Module $m -Force -PassThru
}

function Get-DSCResourceModules {
    $listPSModuleFolders = $env:PSModulePath.Split([IO.Path]::PathSeparator)
    $dscModulePsd1List = [System.Collections.Generic.HashSet[System.String]]::new()
    foreach ($folder in $listPSModuleFolders) {
        if (!(Test-Path $folder)) {
            continue
        }

        foreach ($moduleFolder in Get-ChildItem $folder -Directory) {
            $addModule = $false
            foreach ($psd1 in Get-ChildItem -Recurse -Filter "$($moduleFolder.Name).psd1" -Path $moduleFolder.fullname -Depth 2) {
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
        if (($property -eq $null) -or ($property.IsStatic)) {
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

    if (([System.IO.Path]::GetExtension($filePath) -ne ".psm1") -and ([System.IO.Path]::GetExtension($filePath) -ne ".ps1")) {
        return
    }
    
    "Loading resources from file '$filePath'" | Write-DscTrace -Operation Trace
    #TODO: Ensure embedded instances in properties are working correctly
    [System.Management.Automation.Language.Token[]] $tokens = $null
    [System.Management.Automation.Language.ParseError[]] $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile($filePath, [ref]$tokens, [ref]$errors)
    foreach ($e in $errors) {
        $e | Out-String | Write-DscTrace -Operation Error
    }

    $typeDefinitions = $ast.FindAll(
        {
            $typeAst = $args[0] -as [System.Management.Automation.Language.TypeDefinitionAst]
            return $typeAst -ne $null;
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
                $DscResourceInfo.Operations = GetResourceOperationMethods -resourceName $typeDefinitionAst.Name -filePath $filePath

                $DscResourceInfo.Properties = [System.Collections.Generic.List[DscResourcePropertyInfo]]::new()
                
                Add-AstMembers $typeDefinitions $typeDefinitionAst $DscResourceInfo.Properties
                
                $resourceList.Add($DscResourceInfo)
            }
        }
    }

    return $resourceList
}

function LoadPowerShellClassResourcesFromModule {
    [CmdletBinding(HelpUri = '')]
    param(
        [Parameter(Mandatory = $true)]
        [PSModuleInfo]$moduleInfo
    )

    "Loading resources from module '$($moduleInfo.Path)'" | Write-DscTrace -Operation Trace
    
    if ($moduleInfo.RootModule) {
        if (([System.IO.Path]::GetExtension($moduleInfo.RootModule) -ne ".psm1") -and
            ([System.IO.Path]::GetExtension($moduleInfo.RootModule) -ne ".ps1") -and
            (-not $z.NestedModules)) {
            "RootModule is neither psm1 nor ps1 '$($moduleInfo.RootModule)'" | Write-DscTrace -Operation Trace
            return [System.Collections.Generic.List[DscResourceInfo]]::new()
        }

        $scriptPath = Join-Path $moduleInfo.ModuleBase  $moduleInfo.RootModule
    }
    else {
        $scriptPath = $moduleInfo.Path;
    }

    $Resources = FindAndParseResourceDefinitions $scriptPath $moduleInfo.Version

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

function GetResourceOperationMethods {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string] $resourceName,

        [Parameter(Mandatory = $true)]
        [string] $filePath
    )

    # dot source scope
    try {
        . (LoadClassAndEnumsFromModuleFile -filePath $filePath)
    } catch {
        ("Module: '{0}' not loaded for resource operation discovery."-f $filePath) | Write-DscTrace
    }

    $inputObject = ReturnTypeNameObject -TypeName $resourceName

    if (-not $inputObject) {
        return @(
            'Get',
            'Test',
            'Set'
        )
    }

    # TODO: There might be more properties available
    $knownMemberTypes = @('Equals', 'GetHashCode', 'GetType', 'ToString')
    return ($inputObject | Get-Member | Where-Object { $_.MemberType -eq 'Method' -and $_.Name -notin $knownMemberTypes }).Name
}

function ReturnTypeNameObject {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string] $TypeName
    )

    try {
        $inputObject = New-Object -TypeName $TypeName -ErrorAction Stop
    }
    catch {
        "Could not create: $TypeName" | Write-DscTrace
    }

    return $inputObject
}

function LoadClassAndEnumsFromModuleFile {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [string] $filePath
    )

    if (-not (Test-Path $filePath -ErrorAction SilentlyContinue)) {
        return
    }

    $ctx = Get-Content $filePath

    $string = @(
        'using namespace System.Collections.Generic', # TODO: Figure away out to get using statements included
        (GetEnumCodeBlock -Content $ctx),
        (GetClassCodeBlock -Content $ctx)
    )

    # TODO: Might have to do something with the path
    $outPath = Join-Path -Path $env:TEMP -ChildPath ("{0}.ps1" -f [System.Guid]::NewGuid().Guid)
    $string | Out-File -FilePath $outPath

    return $outPath
}

function GetClassCodeBlock {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [array] $Content
    )

    $ctx = $Content

    $lines = ($ctx | Select-String -Pattern '\[DSCResource\(\)]').LineNumber
    if ($lines.Count -eq 0 ) {
        return
    }

    $lastLineNumber = $lines[-1]
    $index = 1
    # Bring all class strings together after the last one 
    $classStrings = foreach ($line in $lines) {
        if ($line -eq $lastLineNumber) {
            $lastModuleLine = $ctx.Length

            $line = $line - 1
            $block = $ctx[$line..$lastModuleLine]
            $block
            break
        }

        $line = $line - 1
        $curlyBracketLine = FindCurlyBracket -Content $ctx -LineNumber $lines[$index]
        $block = $ctx[$line..$curlyBracketLine]

        $index++
        $block
    }

    return $classStrings
}

function GetEnumCodeBlock {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [array] $Content
    )

    # Build regex to catch enum blocks
    $regex = [regex]::new('enum\s+(\w+)\s*\{([^}]+)\}')

    $hits = $regex.Matches($Content)

    # return as single lines
    return ($hits.Value -Split " ")
}

function FindCurlyBracket {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true)]
        [array] $Content,

        [Parameter(Mandatory = $true)]
        [int] $LineNumber
    )
    do {
        if ($Content[$LineNumber] -eq "}") {
            return $LineNumber
        }

        $LineNumber--
    } while ($LineNumber -ne 0)
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
        "Reading from Get-DscResource cache file $cacheFilePath" | Write-DscTrace

        $cache = Get-Content -Raw $cacheFilePath | ConvertFrom-Json

        if ($cache.CacheSchemaVersion -ne $script:CurrentCacheSchemaVersion) {
            $refreshCache = $true
            "Incompatible version of cache in file '" + $cache.CacheSchemaVersion + "' (expected '" + $script:CurrentCacheSchemaVersion + "')" | Write-DscTrace
        }
        else {
            $dscResourceCacheEntries = $cache.ResourceCache

            if ($dscResourceCacheEntries.Count -eq 0) {
                # if there is nothing in the cache file - refresh cache
                $refreshCache = $true

                "Filtered DscResourceCache cache is empty" | Write-DscTrace
            }
            else {
                "Checking cache for stale entries" | Write-DscTrace

                foreach ($cacheEntry in $dscResourceCacheEntries) {
                    #"Checking cache entry '$($cacheEntry.Type) $($cacheEntry.LastWriteTimes)'" | Write-DscTrace -Operation Trace

                    $cacheEntry.LastWriteTimes.PSObject.Properties | ForEach-Object {
                    
                        if (-not ((Get-Item $_.Name).LastWriteTime.Equals([DateTime]$_.Value))) {
                            "Detected stale cache entry '$($_.Name)'" | Write-DscTrace
                            $refreshCache = $true
                            break
                        }
                    }

                    if ($refreshCache) { break }
                }

                "Checking cache for stale PSModulePath" | Write-DscTrace

                $m = $env:PSModulePath -split [IO.Path]::PathSeparator | % { Get-ChildItem -Directory -Path $_ -Depth 1 -ea SilentlyContinue }

                $hs_cache = [System.Collections.Generic.HashSet[string]]($cache.PSModulePaths)
                $hs_live = [System.Collections.Generic.HashSet[string]]($m.FullName)
                $hs_cache.SymmetricExceptWith($hs_live)
                $diff = $hs_cache

                "PSModulePath diff '$diff'" | Write-DscTrace

                if ($diff.Count -gt 0) {
                    $refreshCache = $true
                }
            }
        }
    }
    else {
        "Cache file not found '$cacheFilePath'" | Write-DscTrace
        $refreshCache = $true
    }
    
    if ($refreshCache) {
        'Constructing Get-DscResource cache' | Write-DscTrace

        # create a list object to store cache of Get-DscResource
        [dscResourceCacheEntry[]]$dscResourceCacheEntries = [System.Collections.Generic.List[Object]]::new()

        $DscResources = [System.Collections.Generic.List[DscResourceInfo]]::new()
        $dscResourceModulePsd1s = Get-DSCResourceModules
        if ($null -ne $dscResourceModulePsd1s) {
            $modules = Get-Module -ListAvailable -Name ($dscResourceModulePsd1s)
            $processedModuleNames = @{}
            foreach ($mod in $modules) {
                if (-not ($processedModuleNames.ContainsKey($mod.Name))) {
                    $processedModuleNames.Add($mod.Name, $true)

                    # from several modules with the same name select the one with the highest version
                    $selectedMod = $modules | Where-Object Name -EQ $mod.Name 
                    if ($selectedMod.Count -gt 1) {
                        "Found $($selectedMod.Count) modules with name '$($mod.Name)'" | Write-DscTrace -Operation Trace
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
            Get-ChildItem -Recurse -File -Path $dscResource.ParentPath -Include "*.ps1", "*.psd1", "*psm1", "*.mof" -ea Ignore | % {
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
        $m = $env:PSModulePath -split [IO.Path]::PathSeparator | % { Get-ChildItem -Directory -Path $_ -Depth 1 -ea SilentlyContinue }
        $cache.PSModulePaths = $m.FullName
        $cache.CacheSchemaVersion = $script:CurrentCacheSchemaVersion

        # save cache for future use
        # TODO: replace this with a high-performance serializer
        "Saving Get-DscResource cache to '$cacheFilePath'" | Write-DscTrace
        $jsonCache = $cache | ConvertTo-Json -Depth 90
        New-Item -Force -Path $cacheFilePath -Value $jsonCache -Type File | Out-Null
    }

    return $dscResourceCacheEntries
}

# Convert the INPUT to a dscResourceObject object so configuration and resource are standardized as much as possible
function Get-DscResourceObject {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        $jsonInput
    )
    # normalize the INPUT object to an array of dscResourceObject objects
    $inputObj = $jsonInput | ConvertFrom-Json
    $desiredState = [System.Collections.Generic.List[Object]]::new()

    # catch potential for improperly formatted configuration input
    if ($inputObj.resources -and -not $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        'WARNING: The input has a top level property named "resources" but is not a configuration. If the input should be a configuration, include the property: "metadata": {"Microsoft.DSC": {"context": "Configuration"}}' | Write-DscTrace
    }

    $adapterName = 'Microsoft.DSC/PowerShell'

    if ($null -ne $inputObj.metadata -and $null -ne $inputObj.metadata.'Microsoft.DSC' -and $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        # change the type from pscustomobject to dscResourceObject
        $inputObj.resources | ForEach-Object -Process {
            $desiredState += [dscResourceObject]@{
                name       = $_.name
                type       = $_.type
                properties = $_.properties
            }
        }
    }
    else {
        # mimic a config object with a single resource
        $type = $inputObj.adapted_dsc_type
        $inputObj.psobject.properties.Remove('adapted_dsc_type')
        $desiredState += [dscResourceObject]@{
            name       = $adapterName
            type       = $type
            properties = $inputObj
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
    'OS version: ' + $osVersion | Write-DscTrace

    $psVersion = $PSVersionTable.PSVersion.ToString()
    'PowerShell version: ' + $psVersion | Write-DscTrace

    # get details from cache about the DSC resource, if it exists
    $cachedDscResourceInfo = $dscResourceCache | Where-Object Type -EQ $DesiredState.type | ForEach-Object DscResourceInfo

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

                    if ($DesiredState.properties) {
                        # set each property of $dscResourceInstance to the value of the property in the $desiredState INPUT object
                        $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                            $dscResourceInstance.$($_.Name) = $_.Value
                        }
                    }

                    switch ($Operation) {
                        'Get' {
                            $Result = $dscResourceInstance.Get()
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
                            $method = $t.GetMethod('Export')
                            $resultArray = $method.Invoke($null, $null)
                            $addToActualState = $resultArray
                        }
                    }
                }
                catch {
                    
                    'ERROR: ' + $_.Exception.Message | Write-DscTrace
                    exit 1
                }
            }
            Default {
                'Resource ImplementationDetail not supported: ' + $cachedDscResourceInfo.ImplementationDetail | Write-DscTrace
                exit 1
            }
        }

        return $addToActualState
    }
    else {
        $dsJSON = $DesiredState | ConvertTo-Json -Depth 10
        $errmsg = 'Can not find type "' + $DesiredState.type + '" for resource "' + $dsJSON + '". Please ensure that Get-DscResource returns this resource type.'
        'ERROR: ' + $errmsg | Write-DscTrace
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

# format expected for configuration and resource output
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
    [System.String[]] $Operations
}
