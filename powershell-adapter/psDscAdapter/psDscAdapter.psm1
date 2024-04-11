# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# if the version of PowerShell is greater than 5, import the PSDesiredStateConfiguration module
# this is necessary because the module is not included in the PowerShell 7.0+ releases
# PSDesiredStateConfiguration 2.0.7 module will be saved in the DSC build
# in Windows PowerShell, we should always use version 1.1 that ships in Windows
if ($PSVersionTable.PSVersion.Major -gt 5) {
    $parentFolder = (Get-Item (Resolve-Path $PSScriptRoot).Path).Parent
    $PSDesiredStateConfiguration = Import-Module "$parentFolder/PSDesiredStateConfiguration/2.0.7/PSDesiredStateConfiguration.psd1" -Force -PassThru
}
else {
    $env:PSModulePath += ";$env:windir\System32\WindowsPowerShell\v1.0\Modules"
    $PSDesiredStateConfiguration = Import-Module -Name 'PSDesiredStateConfiguration' -RequiredVersion '1.1' -Force -PassThru -ErrorAction stop -ErrorVariable $importModuleError
    if (-not [string]::IsNullOrEmpty($importModuleError)) {
        $trace = @{'Debug' = 'ERROR: Could not import PSDesiredStateConfiguration 1.1 in Windows PowerShell. ' + $importModuleError } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
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

    # cache the results of Get-DscResource
    [dscResourceCache[]]$dscResourceCache = [System.Collections.Generic.List[Object]]::new()

    # improve by performance by having the option to only get details for named modules
    # workaround for File and SignatureValidation resources that ship in Windows
    if ($null -ne $module -and 'PSDesiredStateConfiguration' -ne $module) {
        if ($module.gettype().name -eq 'string') {
            $module = @($module)
        }
        $DscResources = [System.Collections.Generic.List[Object]]::new()
        $Modules = [System.Collections.Generic.List[Object]]::new()
        foreach ($m in $module) {
            $DscResources += Get-DscResource -Module $m
            $Modules += Get-Module -Name $m -ListAvailable
        }
    }
    elseif ('PSDesiredStateConfiguration' -eq $module -and $PSVersionTable.PSVersion.Major -le 5 ) {
        # the resources in Windows should only load in Windows PowerShell
        # workaround: the binary modules don't have a module name, so we have to special case File and SignatureValidation resources that ship in Windows
        $DscResources = Get-DscResource | Where-Object { $_.modulename -eq 'PSDesiredStateConfiguration' -or ( $_.modulename -eq $null -and $_.parentpath -like "$env:windir\System32\Configuration\*" ) }
    }
    else {
        # if no module is specified, get all resources
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
                $trace = @{'Debug' = 'WARNING: implementation detail not found: ' + $dscResource.ImplementationDetail } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
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
            }
            else {
                $DscResourceInfo.$($_.Name) = ''
            }
        }

        if ($dscResource.ModuleName) {
            $moduleName = $dscResource.ModuleName
        }
        elseif ($binaryBuiltInModulePaths -contains $dscResource.ParentPath) {
            $moduleName = 'PSDesiredStateConfiguration'
            $DscResourceInfo.Module = 'PSDesiredStateConfiguration'
            $DscResourceInfo.ModuleName = 'PSDesiredStateConfiguration'
            $DscResourceInfo.CompanyName = 'Microsoft Corporation'
            $DscResourceInfo.Version = '1.0.0'
            if ($PSVersionTable.PSVersion.Major -le 5 -and $DscResourceInfo.ImplementedAs -eq 'Binary') {
                $DscResourceInfo.ImplementationDetail = 'Binary'
            }
        }
        elseif ($binaryBuiltInModulePaths -notcontains $dscResource.ParentPath -and $null -ne $dscResource.ParentPath) {
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

        $dscResourceCache += [dscResourceCache]@{
            Type            = "$moduleName/$($dscResource.Name)"
            DscResourceInfo = $DscResourceInfo
        }
    }
    return $dscResourceCache
}

# Convert the INPUT to a dscResourceObject object so configuration and resource are standardized as moch as possible
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
        $trace = @{'Debug' = 'WARNING: The input has a top level property named "resources" but is not a configuration. If the input should be a configuration, include the property: "metadata": {"Microsoft.DSC": {"context": "Configuration"}}' } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
    }

    # match adapter to version of powershell
    if ($PSVersionTable.PSVersion.Major -le 5) {
        $adapterName = 'Microsoft.DSC/WindowsPowerShell'
    }
    else {
        $adapterName = 'Microsoft.DSC/PowerShell'
    }

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
        $type = $inputObj.type
        $inputObj.psobject.properties.Remove('type')
        $desiredState += [dscResourceObject]@{
            name       = $adapterName
            type       = $type
            properties = $inputObj
        }
    }
    return $desiredState
}

# Get the actual state using DSC Get method from any type of DSC resource
function Get-ActualState {
    param(
        [Parameter(Mandatory, ValueFromPipeline = $true)]
        [dscResourceObject]$DesiredState,
        [Parameter(Mandatory)]
        [dscResourceCache[]]$dscResourceCache
    )

    $osVersion = [System.Environment]::OSVersion.VersionString
    $trace = @{'Debug' = 'OS version: ' + $osVersion } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)

    $psVersion = $PSVersionTable.PSVersion.ToString()
    $trace = @{'Debug' = 'PowerShell version: ' + $psVersion } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)

    $moduleVersion = Get-Module PSDesiredStateConfiguration | ForEach-Object Version
    $trace = @{'Debug' = 'PSDesiredStateConfiguration module version: ' + $moduleVersion } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)

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

        $trace = @{'Debug' = 'DSC resource implementation: ' + [dscResourceType]$cachedDscResourceInfo.ImplementationDetail } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        switch ([dscResourceType]$cachedDscResourceInfo.ImplementationDetail) {
            'ScriptBased' {

                # For Linux/MacOS, only class based resources are supported and are called directly.
                if ($IsLinux) {
                    $trace = @{'Debug' = 'ERROR: Script based resources are only supported on Windows.' } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }

                # imports the .psm1 file for the DSC resource as a PowerShell module and stores the list of parameters
                Import-Module -Scope Local -Name $cachedDscResourceInfo.path -Force -ErrorAction stop
                $validParams = (Get-Command -Module $cachedDscResourceInfo.ResourceType -Name 'Get-TargetResource').Parameters.Keys
                # prune any properties that are not valid parameters of Get-TargetResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                    if ($validParams -notcontains $_.Name) {
                        $DesiredState.properties.psobject.properties.Remove($_.Name)
                    }
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

                # using the cmdlet the appropriate dsc module, and handle errors
                try {
                    $getResult = Invoke-DscResource -Method Get -ModuleName $cachedDscResourceInfo.ModuleName -Name $cachedDscResourceInfo.Name -Property $property

                    if ($getResult.GetType().Name -eq 'Hashtable') {
                        $getResult.keys | ForEach-Object -Begin { $getDscResult = @{} } -Process { $getDscResult[$_] = $getResult.$_ }
                    }
                    else {
                        # the object returned by WMI is a CIM instance with a lot of additional data. only return DSC properties
                        $getResult.psobject.Properties.name | Where-Object { 'CimClass', 'CimInstanceProperties', 'CimSystemProperties' -notcontains $_ } | ForEach-Object -Begin { $getDscResult = @{} } -Process { $getDscResult[$_] = $getResult.$_ }
                    }
                    
                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getDscResult
                }
                catch {
                    $trace = @{'Debug' = 'ERROR: ' + $_.Exception.Message } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }
            }
            'ClassBased' {
                try {
                    # load powershell class from external module
                    $resource = GetTypeInstanceFromModule -modulename $cachedDscResourceInfo.ModuleName -classname $cachedDscResourceInfo.Name
                    $dscResourceInstance = $resource::New()

                    # set each property of $dscResourceInstance to the value of the property in the $desiredState INPUT object
                    $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                        $dscResourceInstance.$($_.Name) = $_.Value
                    }
                    $getResult = $dscResourceInstance.Get()

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getResult
                }
                catch {
                    
                    $trace = @{'Debug' = 'ERROR: ' + $_.Exception.Message } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }
            }
            'Binary' {
                if ($PSVersionTable.PSVersion.Major -gt 5) {
                    $trace = @{'Debug' = 'To use a binary resource such as File, Log, or SignatureValidation, use the Microsoft.DSC/WindowsPowerShell adapter.' } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }

                if (-not (($cachedDscResourceInfo.ImplementedAs -eq 'Binary') -and ('File', 'Log', 'SignatureValidation' -contains $cachedDscResourceInfo.Name))) {
                    $trace = @{'Debug' = 'Only File, Log, and SignatureValidation are supported as Binary resources.' } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

                # using the cmdlet from PSDesiredStateConfiguration module in Windows
                try {
                    $getResult = $PSDesiredStateConfiguration.invoke({ param($Name, $Property) Invoke-DscResource -Name $Name -Method Get -ModuleName @{ModuleName = 'PSDesiredStateConfiguration'; ModuleVersion = '1.1' } -Property $Property -ErrorAction Stop }, $cachedDscResourceInfo.Name, $property )

                    if ($getResult.GetType().Name -eq 'Hashtable') {
                        $getResult.keys | ForEach-Object -Begin { $getDscResult = @{} } -Process { $getDscResult[$_] = $getResult.$_ }
                    }
                    else {
                        # the object returned by WMI is a CIM instance with a lot of additional data. only return DSC properties
                        $getResult.psobject.Properties.name | Where-Object { 'CimClass', 'CimInstanceProperties', 'CimSystemProperties' -notcontains $_ } | ForEach-Object -Begin { $getDscResult = @{} } -Process { $getDscResult[$_] = $getResult.$_ }
                    }
                    
                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getDscResult
                }
                catch {
                    $trace = @{'Debug' = 'ERROR: ' + $_.Exception.Message } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }
            }
            Default {
                $trace = @{'Debug' = 'Can not find implementation of type: ' + $cachedDscResourceInfo.ImplementationDetail } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
                exit 1
            }
        }

        return $addToActualState
    }
    else {
        $dsJSON = $DesiredState | ConvertTo-Json -Depth 10
        $errmsg = 'Can not find type "' + $DesiredState.type + '" for resource "' + $dsJSON + '". Please ensure that Get-DscResource returns this resource type.'
        $trace = @{'Debug' = 'ERROR: ' + $errmsg } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
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
class dscResourceCache {
    [string] $Type
    [psobject] $DscResourceInfo
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
}
