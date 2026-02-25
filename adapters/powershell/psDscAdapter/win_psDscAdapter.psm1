# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$global:ProgressPreference = 'SilentlyContinue'
$script:CurrentCacheSchemaVersion = 1

trap {
    Write-DscTrace -Operation Debug -Message ($_ | Format-List -Force | Out-String)
}

function Write-DscTrace {
    param(
        [Parameter(Mandatory = $false)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Operation = 'Debug',

        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{$Operation.ToLower() = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

# if the version of PowerShell is greater than 5, import the PSDesiredStateConfiguration module
# this is necessary because the module is not included in the PowerShell 7.0+ releases;
# In Windows PowerShell, we should always use version 1.1 that ships in Windows.
if ($PSVersionTable.PSVersion.Major -gt 5) {
    $m = Get-Module PSDesiredStateConfiguration -ListAvailable | Sort-Object -Descending | Select-Object -First 1
    $PSDesiredStateConfiguration = Import-Module $m -Force -PassThru
} else {
    $env:PSModulePath = "$env:windir\System32\WindowsPowerShell\v1.0\Modules;$env:PSModulePath"
    $PSDesiredStateConfiguration = Import-Module -Name 'PSDesiredStateConfiguration' -RequiredVersion '1.1' -Force -PassThru -ErrorAction stop -ErrorVariable $importModuleError
    if (-not [string]::IsNullOrEmpty($importModuleError)) {
        'Could not import PSDesiredStateConfiguration 1.1 in Windows PowerShell. ' + $importModuleError | Write-DscTrace -Operation Error
    }
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
    $namedModules = [System.Collections.Generic.List[Object]]::new()
    $cacheFilePath = Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"

    # Repair PSModulePath to not contain empty entries
    Repair-ValidPSModulePath

    if (Test-Path $cacheFilePath) {
        "Reading from Get-DscResource cache file $cacheFilePath" | Write-DscTrace

        $cache = Get-Content -Raw $cacheFilePath | ConvertFrom-Json
        if ($cache.CacheSchemaVersion -ne $script:CurrentCacheSchemaVersion) {
            $refreshCache = $true
            "Incompatible version of cache in file '" + $cache.CacheSchemaVersion + "' (expected '" + $script:CurrentCacheSchemaVersion + "')" | Write-DscTrace
        } else {
            $dscResourceCacheEntries = $cache.ResourceCache

            if ($dscResourceCacheEntries.Count -eq 0) {
                # if there is nothing in the cache file - refresh cache
                $refreshCache = $true
                "Filtered DscResourceCache cache is empty" | Write-DscTrace
            } else {
                "Checking cache for stale PSModulePath" | Write-DscTrace

                $m = $env:PSModulePath -split [IO.Path]::PathSeparator | ForEach-Object { Get-ChildItem -Directory -Path $_ -Depth 1 -ErrorAction Ignore }

                $hs_cache = [System.Collections.Generic.HashSet[string]]($cache.PSModulePaths)
                $hs_live = [System.Collections.Generic.HashSet[string]]($m.FullName)
                $hs_cache.SymmetricExceptWith($hs_live)
                $diff = $hs_cache

                "PSModulePath diff '$diff'" | Write-DscTrace
                # TODO: Optimise for named module refresh
                if ($diff.Count -gt 0) {
                    $refreshCache = $true
                }

                if (-not $refreshCache) {
                    "Checking cache for stale entries" | Write-DscTrace

                    foreach ($cacheEntry in $dscResourceCacheEntries) {

                        foreach ($_ in $cacheEntry.LastWriteTimes.PSObject.Properties) {

                            if (Test-Path $_.Name) {
                                $file_LastWriteTime = (Get-Item $_.Name).LastWriteTime.ToFileTime()
                                $cache_LastWriteTime = [long]$_.Value

                                if ($file_LastWriteTime -ne $cache_LastWriteTime) {
                                    "Detected stale cache entry '$($_.Name)'" | Write-DscTrace
                                    $namedModules.Add($cacheEntry.DscResourceInfo.ModuleName)
                                    break
                                }
                            } else {
                                "Detected non-existent cache entry '$($_.Name)'" | Write-DscTrace
                                $namedModules.Add($cacheEntry.DscResourceInfo.ModuleName)
                                break
                            }
                        }
                    }
                }
                if ($namedModules.Count -gt 0) {
                    $refreshCache = $true
                    if ($null -ne $Module) {
                        $namedModules.AddRange(@($Module))
                    }
                    $namedModules = $namedModules | Sort-Object -Unique
                    "Module list: $($namedModules -join ', ')" | Write-DscTrace
                }
            }
        }
    } else {
        "Cache file not found '$cacheFilePath'" | Write-DscTrace
        $refreshCache = $true
    }

    if ($refreshCache) {
        'Constructing Get-DscResource cache' | Write-DscTrace

        # create a list object to store cache of Get-DscResource
        $dscResourceCacheEntries = [System.Collections.Generic.List[dscResourceCacheEntry]]::new()

        # improve by performance by having the option to only get details for named modules
        # workaround for File and SignatureValidation resources that ship in Windows
        Write-DscTrace -Operation Debug "Named module count: $($namedModules.Count)"
        if ($namedModules.Count -gt 0) {
            Write-DscTrace -Operation Debug "Modules specified, getting DSC resources from modules: $($namedModules -join ', ')"
            $DscResources = [System.Collections.Generic.List[Object]]::new()
            $Modules = [System.Collections.Generic.List[Object]]::new()
            $filteredResources = @()
            foreach ($m in $namedModules) {
                Write-DscTrace -Operation Debug "Getting DSC resources for module '$($m | Out-String)'"
                $DscResources.AddRange(@(Get-DscResource -Module $m))
                $Modules.AddRange(@(Get-Module -Name $m -ListAvailable))
            }

            if ('PSDesiredStateConfiguration' -in $namedModules -and $PSVersionTable.PSVersion.Major -le 5 ) {
                # the resources in Windows should only load in Windows PowerShell
                # workaround: the binary modules don't have a module name, so we have to special case File and SignatureValidation resources that ship in Windows
                $DscResources.AddRange(@(Get-DscResource | Where-Object -Property ParentPath -eq "$env:windir\system32\Configuration\BaseRegistration"))
                $filteredResources = @(
                    'PSDesiredStateConfiguration/File'
                    'PSDesiredStateConfiguration/SignatureValidation'
                )
            }
            # Grab all DSC resources to filter out of the cache
            $filteredResources += $dscResources | Where-Object -Property ModuleName -NE $null | ForEach-Object { [System.String]::Concat($_.ModuleName, '/', $_.Name) }
            # Exclude the one module that was passed in as a parameter
            $existingDscResourceCacheEntries = @($cache.ResourceCache | Where-Object -Property Type -NotIn $filteredResources)
        } else {
            Write-DscTrace -Operation Debug "No modules specified, getting all DSC resources"
            $DscResources = Get-DscResource
            $Modules = Get-Module -ListAvailable
        }

        $psdscVersion = Get-Module PSDesiredStateConfiguration | Sort-Object -descending | Select-Object -First 1 | ForEach-Object Version

        foreach ($dscResource in $DscResources) {
            # resources that shipped in Windows should only be used with Windows PowerShell
            if ($dscResource.ParentPath -like "$env:windir\System32\*" -and $PSVersionTable.PSVersion.Major -gt 5) {
                continue
            }

            # we can't run this check in PSDesiredStateConfiguration 1.1 because the property doesn't exist
            if ( $psdscVersion -ge '2.0.7' ) {
                # only support known dscResourceType
                if ([dscResourceType].GetEnumNames() -notcontains $dscResource.ImplementationDetail) {
                    'Implementation detail not found: ' + $dscResource.ImplementationDetail | Write-DscTrace -Operation Warn
                    continue
                }
            }

            # workaround: if the resource does not have a module name, get it from parent path
            # workaround: modulename is not settable, so clone the object without being read-only
            # workaround: we have to special case File and SignatureValidation resources that ship in Windows
            $binaryBuiltInModulePaths = @(
                "$env:windir\system32\Configuration\Schema\MSFT_FileDirectoryConfiguration"
                "$env:windir\system32\Configuration\BaseRegistration"
            )
            $DscResourceInfo = [DscResourceInfo]::new()
            $dscResource.PSObject.Properties | ForEach-Object -Process {
                if ($null -ne $_.Value) {
                    $DscResourceInfo.$($_.Name) = $_.Value
                } else {
                    $DscResourceInfo.$($_.Name) = ''
                }
            }

            if ($dscResource.ModuleName) {
                $moduleName = $dscResource.ModuleName
            } elseif ($binaryBuiltInModulePaths -contains $dscResource.ParentPath) {
                $moduleName = 'PSDesiredStateConfiguration'
                $DscResourceInfo.Module = 'PSDesiredStateConfiguration'
                $DscResourceInfo.ModuleName = 'PSDesiredStateConfiguration'
                $DscResourceInfo.CompanyName = 'Microsoft Corporation'
                $DscResourceInfo.Version = '1.0.0'
                if ($PSVersionTable.PSVersion.Major -le 5 -and $DscResourceInfo.ImplementedAs -eq 'Binary') {
                    $DscResourceInfo.ImplementationDetail = 'Binary'
                }
            } elseif ($binaryBuiltInModulePaths -notcontains $dscResource.ParentPath -and $null -ne $dscResource.ParentPath) {
                # workaround: populate module name from parent path that is three levels up
                $moduleName = Split-Path $dscResource.ParentPath | Split-Path | Split-Path -Leaf
                $DscResourceInfo.Module = $moduleName
                $DscResourceInfo.ModuleName = $moduleName
                # workaround: populate module version from psmoduleinfo if available
                if ($moduleInfo = $Modules | Where-Object { $_.Name -eq $moduleName }) {
                    $moduleInfo = $moduleInfo | Sort-Object -Property Version -Descending | Select-Object -First 1
                    $DscResourceInfo.Version = $moduleInfo.Version.ToString()
                }
            }

            # workaround: Use GetTypeInstanceFromModule to get the type instance from the module and validate if it is a class-based resource
            $classBased = GetTypeInstanceFromModule -modulename $moduleName -classname $dscResource.Name -ErrorAction Ignore
            if ($classBased -and ($classBased.CustomAttributes.AttributeType.Name -eq 'DscResourceAttribute')) {
                "Detected class-based resource: $($dscResource.Name) => Type: $($classBased.BaseType.FullName)" | Write-DscTrace
                $dscResourceInfo.ImplementationDetail = 'ClassBased'
                $properties = GetClassBasedProperties -filePath $dscResource.Path -className $dscResource.Name
                if ($null -ne $properties) {
                    $DscResourceInfo.Properties = $properties
                }

                $dscResourceInfo.Capabilities = GetClassBasedCapabilities -filePath $dscResource.Path -className $dscResource.Name
            }

            # fill in resource files (and their last-write-times) that will be used for up-do-date checks
            $lastWriteTimes = @{}
            Get-ChildItem -Recurse -File -Path $dscResource.ParentPath -Include "*.ps1", "*.psd1", "*.psm1", "*.mof" -ea Ignore | ForEach-Object {
                $lastWriteTimes.Add($_.FullName, $_.LastWriteTime.ToFileTime())
            }

            $dscResourceCacheEntries.Add([dscResourceCacheEntry]@{
                Type            = "$moduleName/$($dscResource.Name)"
                DscResourceInfo = $DscResourceInfo
                LastWriteTimes  = $lastWriteTimes
            })
        }

        if ($namedModules.Count -gt 0) {
            # Make sure all resource cache entries are returned
            foreach ($entry in $existingDscResourceCacheEntries) {
                $dscResourceCacheEntries.Add([dscResourceCacheEntry]$entry)
            }
        }

        [dscResourceCache]$cache = [dscResourceCache]::new()
        $cache.ResourceCache = $dscResourceCacheEntries.ToArray()
        $m = $env:PSModulePath -split [IO.Path]::PathSeparator | ForEach-Object { Get-ChildItem -Directory -Path $_ -Depth 1 -ea SilentlyContinue }
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
    'OS version: ' + $osVersion | Write-DscTrace

    $psVersion = $PSVersionTable.PSVersion.ToString()
    'PowerShell version: ' + $psVersion | Write-DscTrace

    $moduleVersion = Get-Module PSDesiredStateConfiguration | ForEach-Object Version
    'PSDesiredStateConfiguration module version: ' + $moduleVersion | Write-DscTrace

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

        'DSC resource implementation: ' + [dscResourceType]$cachedDscResourceInfo.ImplementationDetail | Write-DscTrace

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        switch ([dscResourceType]$cachedDscResourceInfo.ImplementationDetail) {
            'ScriptBased' {

                # For Linux/MacOS, only class based resources are supported and are called directly.
                if ($IsLinux) {
                    'Script based resources are only supported on Windows.' | Write-DscTrace -Operation Error
                    exit 1
                }

                # imports the .psm1 file for the DSC resource as a PowerShell module and stores the list of parameters
                Import-Module -Scope Local -Name $cachedDscResourceInfo.path -Force -ErrorAction stop
                $validParams = (Get-Command -Module $cachedDscResourceInfo.ResourceType -Name 'Get-TargetResource').Parameters.Keys

                if ($Operation -eq 'Get') {
                    # prune any properties that are not valid parameters of Get-TargetResource
                    $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                        if ($validParams -notcontains $_.Name) {
                            $DesiredState.properties.psobject.properties.Remove($_.Name)
                        }
                    }
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process {
                    if ($_.Value -is [System.Management.Automation.PSCustomObject]) {
                        $validateProperty = $cachedDscResourceInfo.Properties | Where-Object -Property Name -EQ $_.Name
                        Write-DscTrace -Operation Debug -Message "Property type: $($validateProperty.PropertyType)"
                        if ($validateProperty -and $validateProperty.PropertyType -eq '[PSCredential]') {

                            $hasSecureCred =
                                $_.Value.PSObject.Properties['secureObject'] -and
                                $_.Value.secureObject.Username -and
                                $_.Value.secureObject.Password

                            $hasTextCred =
                                $_.Value.Username -and
                                $_.Value.Password

                            if (-not $hasSecureCred -and -not $hasTextCred) {
                                "Credential object '$($_.Name)' requires both 'username' and 'password' properties" |
                                    Write-DscTrace -Operation Error
                                exit 1
                            }

                            if ($hasSecureCred) {
                                "Credential object '$($_.Name)' - SecureObject" | Write-DscTrace -Operation Info

                                $username = $_.Value.secureObject.Username
                                $password = $_.Value.secureObject.Password |
                                    ConvertTo-SecureString -AsPlainText -Force

                                $property.$($_.Name) =
                                    [System.Management.Automation.PSCredential]::new($username, $password)
                            }
                            elseif ($hasTextCred) {
                                "Credential object '$($_.Name)' - Text" | Write-DscTrace -Operation Info

                                $username = $_.Value.Username
                                $password = $_.Value.Password |
                                    ConvertTo-SecureString -AsPlainText -Force

                                $property.$($_.Name) =
                                    [System.Management.Automation.PSCredential]::new($username, $password)
                            }
                        } else {
                            $property.$($_.Name) = $_.Value.psobject.properties | ForEach-Object -Begin { $propertyHash = @{} } -Process { $propertyHash[$_.Name] = $_.Value } -End { $propertyHash }
                        }
                    } else {
                        $property[$_.Name] = $_.Value
                    }
                }

                # using the cmdlet the appropriate dsc module, and handle errors
                try {
                    Write-DscTrace -Operation Debug -Message "Module: $($cachedDscResourceInfo.ModuleName), Name: $($cachedDscResourceInfo.Name), Property: $($property | ConvertTo-Json -Compress)"
                    $invokeResult = Invoke-DscResource -Method $Operation -ModuleName $cachedDscResourceInfo.ModuleName -Name $cachedDscResourceInfo.Name -Property $property -ErrorAction Stop

                    if ($invokeResult.GetType().Name -eq 'Hashtable') {
                        $invokeResult.keys | ForEach-Object -Begin { $ResultProperties = @{} } -Process { $ResultProperties[$_] = $invokeResult.$_ }
                    } else {
                        # the object returned by WMI is a CIM instance with a lot of additional data. only return DSC properties
                        $invokeResult.psobject.Properties.name | Where-Object { 'CimClass', 'CimInstanceProperties', 'CimSystemProperties' -notcontains $_ } | ForEach-Object -Begin { $ResultProperties = @{} } -Process { $ResultProperties[$_] = $invokeResult.$_ }
                    }

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $ResultProperties
                } catch {
                    $_.Exception | Format-List * -Force | Out-String | Write-DscTrace -Operation Debug
                    if ($_.Exception.MessageId -eq 'DscResourceNotFound') {
                        Write-DscTrace -Operation Warn -Message 'For Windows PowerShell, DSC resources must be installed with scope AllUsers'
                    }
                    'Exception: ' + $_.Exception.Message | Write-DscTrace -Operation Error
                    exit 1
                }
            }
            'ClassBased' {
                try {
                    # load powershell class from external module
                    $resource = GetTypeInstanceFromModule -modulename $cachedDscResourceInfo.ModuleName -classname $cachedDscResourceInfo.Name
                    $dscResourceInstance = $resource::New()

                    $ValidProperties = $cachedDscResourceInfo.Properties.Name

                    $ValidProperties | ConvertTo-Json | Write-DscTrace -Operation Trace

                    if ($DesiredState.properties) {
                        # set each property of $dscResourceInstance to the value of the property in the $desiredState INPUT object
                        $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                            # handle input objects by converting them to a hash table
                            if ($_.Value -is [System.Management.Automation.PSCustomObject]) {
                                $validateProperty = $cachedDscResourceInfo.Properties | Where-Object -Property Name -EQ $_.Name
                                Write-DscTrace -Operation Debug -Message "Property type: $($validateProperty.PropertyType)"
                                if ($validateProperty.PropertyType -eq 'PSCredential') {
                                $hasSecureCred =
                                    $_.Value.secureObject.Username -and
                                    $_.Value.secureObject.Password

                                $hasTextCred =
                                    $_.Value.Username -and
                                    $_.Value.Password

                                if (-not $hasSecureCred -and -not $hasTextCred) {
                                    "Invalid credential object for property '$($_.Name)'" | Write-DscTrace -Operation Warn
                                    "Credential object '$($_.Name)' requires both 'username' and 'password' properties" |
                                        Write-DscTrace -Operation Error
                                    exit 1
                                }

                                if ($hasSecureCred) {
                                "Credential object '$($_.Name)' - SecureObject" | Write-DscTrace -Operation Info

                                    $username = $_.Value.secureObject.Username
                                    $password = $_.Value.secureObject.Password |
                                        ConvertTo-SecureString -AsPlainText -Force

                                    $dscResourceInstance.$($_.Name) =
                                        [System.Management.Automation.PSCredential]::new($username, $password)
                                }
                                elseif ($hasTextCred) {
                                    "Credential object '$($_.Name)' - Text" | Write-DscTrace -Operation Info

                                    $username = $_.Value.Username
                                    $password = $_.Value.Password |
                                        ConvertTo-SecureString -AsPlainText -Force

                                    $dscResourceInstance.$($_.Name) =
                                        [System.Management.Automation.PSCredential]::new($username, $password)
                                }

                                } else {
                                    $dscResourceInstance.$($_.Name) = $_.Value.psobject.properties | ForEach-Object -Begin { $propertyHash = @{} } -Process { $propertyHash[$_.Name] = $_.Value } -End { $propertyHash }
                                }
                            } else {
                                $dscResourceInstance.$($_.Name) = $_.Value
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
                                } else {
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
                            $method = ValidateMethod -operation $Operation -class $dscResourceInstance
                            $resultArray = @()
                            $raw_obj_array = $method.Invoke($null, $null)
                            foreach ($raw_obj in $raw_obj_array) {
                                $Result_obj = @{}
                                $ValidProperties | ForEach-Object {
                                    if ($raw_obj.$_ -is [System.Enum]) {
                                        $Result_obj[$_] = $raw_obj.$_.ToString()
                                    } else {
                                        $Result_obj[$_] = $raw_obj.$_
                                    }
                                }
                                $resultArray += $Result_obj
                            }
                            $addToActualState = $resultArray
                        }
                    }
                } catch {
                    $_.Exception | Format-List * -Force | Out-String | Write-DscTrace -Operation Debug
                    if ($_.Exception.MessageId -eq 'DscResourceNotFound') {
                        Write-DscTrace -Operation Warn -Message 'For Windows PowerShell, DSC resources must be installed with scope AllUsers'
                    }
                    'Exception: ' + $_.Exception.Message | Write-DscTrace -Operation Error
                    exit 1
                }
            }
            'Binary' {
                if ($PSVersionTable.PSVersion.Major -gt 5) {
                    'To use a binary resource such as File, Log, or SignatureValidation, use the Microsoft.Windows/WindowsPowerShell adapter.' | Write-DscTrace
                    exit 1
                }

                if (-not (($cachedDscResourceInfo.ImplementedAs -eq 'Binary') -and ('File', 'Log', 'SignatureValidation' -contains $cachedDscResourceInfo.Name))) {
                    'Only File, Log, and SignatureValidation are supported as Binary resources.' | Write-DscTrace
                    exit 1
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process {
                    if ($_.Value -is [System.Management.Automation.PSCustomObject]) {
                        $validateProperty = $cachedDscResourceInfo.Properties | Where-Object -Property Name -EQ $_.Name
                        Write-DscTrace -Operation Debug -Message "Property type: $($validateProperty.PropertyType)"
                        if ($validateProperty.PropertyType -eq '[PSCredential]') {
                            if (-not $_.Value.Username -or -not $_.Value.Password) {
                                "Credential object '$($_.Name)' requires both 'username' and 'password' properties" | Write-DscTrace -Operation Error
                                exit 1
                            }
                            $property.$($_.Name) = [System.Management.Automation.PSCredential]::new($_.Value.Username, (ConvertTo-SecureString -AsPlainText $_.Value.Password -Force))
                        } else {
                            $property.$($_.Name) = $_.Value.psobject.properties | ForEach-Object -Begin { $propertyHash = @{} } -Process { $propertyHash[$_.Name] = $_.Value } -End { $propertyHash }
                        }
                    } else {
                        $property[$_.Name] = $_.Value
                    }
                }

                # using the cmdlet from PSDesiredStateConfiguration module in Windows
                try {
                    Write-DscTrace -Operation Debug -Message "Module: $($cachedDscResourceInfo.ModuleName), Name: $($cachedDscResourceInfo.Name), Property: $($property | ConvertTo-Json -Compress)"
                    $invokeResult = Invoke-DscResource -Method $Operation -ModuleName $cachedDscResourceInfo.ModuleName -Name $cachedDscResourceInfo.Name -Property $property
                    if ($invokeResult.GetType().Name -eq 'Hashtable') {
                        $invokeResult.keys | ForEach-Object -Begin { $ResultProperties = @{} } -Process { $ResultProperties[$_] = $invokeResult.$_ }
                    } else {
                        # the object returned by WMI is a CIM instance with a lot of additional data. only return DSC properties
                        $invokeResult.psobject.Properties.name | Where-Object { 'CimClass', 'CimInstanceProperties', 'CimSystemProperties' -notcontains $_ } | ForEach-Object -Begin { $ResultProperties = @{} } -Process { $ResultProperties[$_] = $invokeResult.$_ }
                    }

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $ResultProperties
                } catch {
                    'Exception: ' + $_.Exception.Message | Write-DscTrace -Operation Error
                    exit 1
                }
            }
            Default {
                'Can not find implementation of type: ' + $cachedDscResourceInfo.ImplementationDetail | Write-DscTrace
                exit 1
            }
        }

        return $addToActualState
    } else {
        $dsJSON = $DesiredState | ConvertTo-Json -Depth 10
        'Can not find type "' + $DesiredState.type + '" for resource "' + $dsJSON + '". Please ensure that Get-DscResource returns this resource type.' | Write-DscTrace -Operation Error
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

# ValidateMethod checks if the specified method exists in the class
function ValidateMethod {
    param (
        [Parameter(Mandatory = $true)]
        [ValidateSet('Export', 'WhatIf')]
        [string] $operation,
        [Parameter(Mandatory = $true)]
        [object] $class
    )

    $t = $class.GetType()
    $methods = $t.GetMethods() | Where-Object -Property Name -EQ $operation
    $method = foreach ($mt in $methods) {
        if ($mt.GetParameters().Count -eq 0) {
            $mt
            break
        }
    }

    if ($null -eq $method) {
        "Method '$operation' not implemented by resource '$($t.Name)'" | Write-DscTrace -Operation Error
        exit 1
    }

    return $method
}

function GetClassBasedProperties {
    param (
        [Parameter(Mandatory = $true)]
        [string] $filePath,

        [Parameter(Mandatory = $true)]
        [string] $className
    )

    if (".psd1" -notcontains ([System.IO.Path]::GetExtension($filePath))) {
        return @('get', 'set', 'test')
    }

    $module = Import-Module $filePath -PassThru -Force -ErrorAction Ignore

    $properties = [System.Collections.Generic.List[DscResourcePropertyInfo]]::new()

    if (Test-Path $module.Path -ErrorAction Ignore) {
        [System.Management.Automation.Language.Token[]] $tokens = $null
        [System.Management.Automation.Language.ParseError[]] $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile($module.Path, [ref]$tokens, [ref]$errors)
        foreach ($e in $errors) {
            $e | Out-String | Write-DscTrace -Operation Warn
        }

        $typeDefinitions = $ast.FindAll(
            {
                $typeAst = $args[0] -as [System.Management.Automation.Language.TypeDefinitionAst]
                return $null -ne $typeAst;
            },
            $false);

        $typeDefinition = $typeDefinitions | Where-Object -Property Name -EQ $className

        foreach ($member in $typeDefinition.Members) {
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
            $properties.Add($prop)
        }
        return $properties
    }
}

function GetClassBasedCapabilities {
    param (
        [Parameter(Mandatory = $true)]
        [string] $filePath,

        [Parameter(Mandatory = $true)]
        [string] $className
    )

    if (".psd1" -notcontains ([System.IO.Path]::GetExtension($filePath))) {
        return @('get', 'set', 'test')
    }

    $module = $filePath.Replace('.psd1', '.psm1')

    if (Test-Path $module -ErrorAction Ignore) {
        [System.Management.Automation.Language.Token[]] $tokens = $null
        [System.Management.Automation.Language.ParseError[]] $errors = $null
        $ast = [System.Management.Automation.Language.Parser]::ParseFile($module, [ref]$tokens, [ref]$errors)
        foreach ($e in $errors) {
            $e | Out-String | Write-DscTrace -Operation Error
        }

        $typeDefinitions = $ast.FindAll(
            {
                $typeAst = $args[0] -as [System.Management.Automation.Language.TypeDefinitionAst]
                return $null -ne $typeAst;
            },
            $false);


        $capabilities = [System.Collections.Generic.List[string[]]]::new()
        $availableMethods = @('get', 'set', 'setHandlesExist', 'whatIf', 'test', 'delete', 'export')
        foreach ($typeDefinitionAst in $typeDefinitions) {
            foreach ($a in $typeDefinitionAst.Attributes) {
                if ($a.TypeName.Name -eq 'DscResource' -and $a.Parent.Name -eq $className) {
                    $methods = $typeDefinitionAst.Members | Where-Object { $_ -is [System.Management.Automation.Language.FunctionMemberAst] -and $_.Name -in $availableMethods }

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
                }
            }
        }

        return $capabilities
    }
}

function Repair-ValidPSModulePath {
    [CmdletBinding()]
    param()

    end {
        if (($env:PSModulePath -split [System.IO.Path]::PathSeparator) -contains '') {
            "Removing empty entry from PSModulePath: '$env:PSModulePath'" | Write-DscTrace -Operation Debug
            $env:PSModulePath = [String]::Join([System.IO.Path]::PathSeparator, ($env:PSModulePath.Split([System.IO.Path]::PathSeparator, [System.StringSplitOptions]::RemoveEmptyEntries))).TrimEnd([System.IO.Path]::PathSeparator)
        }
    }
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
    [psobject[]] $Properties
    [string[]] $Capabilities
}