# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate, ClearCache.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate', 'ClearCache')]
    [string]$Operation,
    [Parameter(Mandatory = $false, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '@{}',
    [Parameter(Mandatory = $false)]
    [string]$ResourceType,
    [Parameter(Mandatory = $false)]
    [string[]]$ResourcePath
)

Import-Module -Name "$PSScriptRoot/psadapter_helpers.psm1" -Force

switch ($Operation) {
    'List' {
        $dscResourceCache = Invoke-DscCacheRefresh

        # cache was refreshed on script load
        foreach ($dscResource in $dscResourceCache.Values) {

            # https://learn.microsoft.com/dotnet/api/system.management.automation.dscresourceinfo
            $DscResourceInfo = $dscResource.DscResourceInfo

            # Provide a way for existing resources to specify their capabilities, or default to Get, Set, Test
            # TODO: for perf, it is better to take capabilities from psd1 in Invoke-DscCacheRefresh, not by extra call to Get-Module
            if ($DscResourceInfo.ModuleName) {
                $module = Get-Module -Name $DscResourceInfo.ModuleName -ListAvailable | Sort-Object -Property Version -Descending | Select-Object -First 1
                # If the DscResourceInfo does have capabilities, use them or else use the module's capabilities
                if ($DscResourceInfo.Capabilities) {
                    $capabilities = $DscResourceInfo.Capabilities
                } elseif ($module.PrivateData.PSData.DscCapabilities) {

                    $capabilities = $module.PrivateData.PSData.DscCapabilities
                } else {
                    $capabilities = @('get', 'set', 'test')
                }
            }

            # this text comes directly from the resource manifest for v3 native resources
            if ($DscResourceInfo.Description) {
                $description = $DscResourceInfo.Description
            }
            elseif ($module.Description) {
                # some modules have long multi-line descriptions. to avoid issue, use only the first line.
                $description = $module.Description.split("`r`n")[0]
            }
            else {
                $description = ''
            }

            # match adapter to version of powershell
            if ($PSVersionTable.PSVersion.Major -le 5) {
                $requireAdapter = 'Microsoft.Windows/WindowsPowerShell'
            }
            else {
                $requireAdapter = 'Microsoft.DSC/PowerShell'
            }

            # OUTPUT dsc is expecting the following properties
            [resourceOutput]@{
                type           = $dscResource.Type
                kind           = 'resource'
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
    { @('Get','Set','Test','Export') -contains $_ } {
        $ds = $jsonInput | ConvertFrom-Json

        # if ResourcePath is provided, we load that module
        if ($ResourcePath) {
            $module = Import-Module -Name $ResourcePath -Force -ErrorAction Stop -PassThru
        } else {
            # refresh the cache with the modules that are available on the system
            $dscResourceCache = Invoke-DscCacheRefresh -module $dscResourceModules
            if (!$dscResourceCache.ContainsKey($ResourceType)) {
                Write-DscTrace -Level Error -Message "DSC resource type '$ResourceType' not found."
                exit 1
            }
        }

        # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
        $actualState = $psDscAdapter.invoke( { param($op, $ds, $dscResourceCache) Invoke-DscOperation -Operation $op -DesiredState $ds -dscResourceCache $dscResourceCache }, $Operation, $ds, $dscResourceCache)
        if ($null -eq $actualState) {
            Write-DscTrace -Level Error -Message 'Incomplete GET for resource ' + $ds.Name
            exit 1
        }
        if ($null -ne $actualState.Properties -and $actualState.Properties.InDesiredState -eq $false) {
            $inDesiredState = $false
        }

        # OUTPUT json to stderr for debug, and to stdout
        if ($Operation -eq 'Test') {
            $result = @{ result = $result; _inDesiredState = $inDesiredState } | ConvertTo-Json -Depth 10 -Compress
        }
        else {
            $result = @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
        }
        Write-DscTrace -Level Debug -Message "jsonOutput=$result"
        return $result        
    }
    'Validate' {
        # VALIDATE not implemented

        # OUTPUT
        @{ valid = $true } | ConvertTo-Json
    }
    Default {
        Write-DscTrace -Level Error -Message 'Unsupported operation. Please use one of the following: List, Get, Set, Test, Export, Validate'
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
