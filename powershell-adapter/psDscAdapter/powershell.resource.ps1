# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate', 'Schema', 'ClearCache')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '@{}'
)

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

# Adding some debug info to STDERR
'PSVersion=' + $PSVersionTable.PSVersion.ToString() | Write-DscTrace
'PSPath=' + $PSHome | Write-DscTrace
'PSModulePath=' + $env:PSModulePath | Write-DscTrace

$cacheFilePath = if ($IsWindows) {
    # PS 6+ on Windows
    Join-Path $env:LocalAppData "dsc\PSAdapterCache.json"
} else {
    # either WinPS or PS 6+ on Linux/Mac
    if ($PSVersionTable.PSVersion.Major -le 5) {
        Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"
    } else {
        Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
    }
}

if ($Operation -eq 'ClearCache') {
    'Deleting cache file ' + $cacheFilePath | Write-DscTrace
    Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath
    exit 0
}

if ('Validate' -ne $Operation) {
    # write $jsonInput to STDERR for debugging
    $trace = @{'Debug' = 'jsonInput=' + $jsonInput } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)

    # load private functions of psDscAdapter stub module
    if ($PSVersionTable.PSVersion.Major -le 5) {
        $psDscAdapter = Import-Module "$PSScriptRoot/win_psDscAdapter.psd1" -Force -PassThru
    } else {
        $psDscAdapter = Import-Module "$PSScriptRoot/psDscAdapter.psd1" -Force -PassThru
    }

    # initialize OUTPUT as array
    $result = [System.Collections.Generic.List[Object]]::new()
}

if ($jsonInput) {
    if ($jsonInput -ne '@{}') {
        $inputobj_pscustomobj = $jsonInput | ConvertFrom-Json
    }
    $new_psmodulepath = $inputobj_pscustomobj.psmodulepath
    if ($new_psmodulepath) {
        $env:PSModulePath = $ExecutionContext.InvokeCommand.ExpandString($new_psmodulepath)
    }
}

# process the operation requested to the script
switch ($Operation) {
    'List' {
        $dscResourceCache = Invoke-DscCacheRefresh

        # cache was refreshed on script load
        foreach ($dscResource in $dscResourceCache) {

            # https://learn.microsoft.com/dotnet/api/system.management.automation.dscresourceinfo
            $DscResourceInfo = $dscResource.DscResourceInfo

            # Provide a way for existing resources to specify their capabilities, or default to Get, Set, Test
            # TODO: for perf, it is better to take capabilities from psd1 in Invoke-DscCacheRefresh, not by extra call to Get-Module
            if ($DscResourceInfo.ModuleName) {
                $module = Get-Module -Name $DscResourceInfo.ModuleName -ListAvailable | Sort-Object -Property Version -Descending | Select-Object -First 1
                if ($module.PrivateData.PSData.DscCapabilities) {
                    $capabilities = $module.PrivateData.PSData.DscCapabilities
                } else {
                    $capabilities = @('Get', 'Set', 'Test')
                }
            }

            # this text comes directly from the resource manifest for v3 native resources
            if ($DscResourceInfo.Description) {
                $description = $DscResourceInfo.Description
            } elseif ($module.Description) {
                # some modules have long multi-line descriptions. to avoid issue, use only the first line.
                $description = $module.Description.split("`r`n")[0]
            } else {
                $description = ''
            }

            # match adapter to version of powershell
            if ($PSVersionTable.PSVersion.Major -le 5) {
                $requireAdapter = 'Microsoft.Windows/WindowsPowerShell'
            } else {
                $requireAdapter = 'Microsoft.DSC/PowerShell'
            }

            # OUTPUT dsc is expecting the following properties
            [resourceOutput]@{
                type           = $dscResource.Type
                kind           = 'Resource'
                version        = [string]$DscResourceInfo.version
                capabilities   = $capabilities
                path           = $DscResourceInfo.Path
                directory      = $DscResourceInfo.ParentPath
                implementedAs  = $DscResourceInfo.ImplementationDetail
                author         = $DscResourceInfo.CompanyName
                properties     = $DscResourceInfo.Properties.Name
                requireAdapter = $requireAdapter
                description    = $description
            } | ConvertTo-Json -Compress
        }
    }
    { @('Get', 'Set', 'Test', 'Export') -contains $_ } {
        $desiredState = $psDscAdapter.invoke(   { param($jsonInput) Get-DscResourceObject -jsonInput $jsonInput }, $jsonInput )
        if ($null -eq $desiredState) {
            $trace = @{'Debug' = 'ERROR: Failed to create configuration object from provided input JSON.' } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
            exit 1
        }

        # only need to cache the resources that are used
        $dscResourceModules = $desiredState | ForEach-Object { $_.Type.Split('/')[0] }
        if ($null -eq $dscResourceModules) {
            $trace = @{'Debug' = 'ERROR: Could not get list of DSC resource types from provided JSON.' } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
            exit 1
        }

        $dscResourceCache = Invoke-DscCacheRefresh -module $dscResourceModules
        if ($dscResourceCache.count -lt $dscResourceModules.count) {
            $trace = @{'Debug' = 'ERROR: DSC resource module not found.' } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
            exit 1
        }

        foreach ($ds in $desiredState) {
            # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
            $actualState = $psDscAdapter.invoke( { param($op, $ds, $dscResourceCache) Invoke-DscOperation -Operation $op -DesiredState $ds -dscResourceCache $dscResourceCache }, $Operation, $ds, $dscResourceCache)
            if ($null -eq $actualState) {
                $trace = @{'Debug' = 'ERROR: Incomplete GET for resource ' + $ds.Name } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
                exit 1
            }
            $result += $actualState
        }

        # OUTPUT json to stderr for debug, and to stdout
        $result = @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
        $trace = @{'Debug' = 'jsonOutput=' + $result } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
        return $result
    }
    'Schema' {
        $cache = Get-Content $cacheFilePath | ConvertFrom-Json

        # TODO: Validate how input is passed and remove hindden properties
        $resourceInfoproperties = ($cache.ResourceCache | Where-Object { $_.Type -eq 'TestClassResource/TestClassResource' }).DscResourceInfo.Properties

        $props = @{}
        $resourceInfoproperties | Foreach-Object {
            if ($_.IsMandatory -eq $true) {
                $props[$_.Name] = [hashtable]@{
                    type        = $_.PropertyType
                    description = ""
                }
            } else {
                $props[$_.Name] = [hashtable]@{
                    type        = @($_.PropertyType, $null)
                    description = ""
                }
            }
        }

        $out = [resourceProperties]@{
            schema               = 'http://json-schema.org/draft-04/schema#'
            title                = ($cache.ResourceCache | Where-Object { $_.Type -eq 'TestClassResource/TestClassResource' }).Type
            type                 = 'object'
            required             = @($resourceInfoproperties | Where-Object { $_.IsMandatory -eq $true }).Name
            properties           = $props
            additionalProperties = $false
            # definitions = $null # TODO: Should we add definitions
        }

        $out | ConvertTo-Json -Depth 10 -Compress
    }
    'Validate' {
        # VALIDATE not implemented

        # OUTPUT
        @{ valid = $true } | ConvertTo-Json
    }
    Default {
        Write-Error 'Unsupported operation. Please use one of the following: List, Get, Set, Test, Export, Schema, Validate'
    }
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

class resourceProperties {
    [string] $schema
    [string] $title
    [string] $type
    [string[]] $required
    [hashtable] $properties
    [bool] $additionalProperties
}