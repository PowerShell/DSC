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

if ('Validate'-ne $Operation) {
    # write $jsonInput to STDERR for debugging
    $trace = @{'Debug' = 'jsonInput=' + $jsonInput } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)

    # load private functions of psDscAdapter stub module
    $psDscAdapter = Import-Module "$PSScriptRoot/psDscAdapter/psDscAdapter.psd1" -Force -PassThru

    # initialize OUTPUT as array
    $result = [System.Collections.Generic.List[Object]]::new()
}

# process the operation requested to the script
switch ($Operation) {
    'List' {
        $resourceCache = Invoke-DscCacheRefresh

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
        $desiredState = $psDscAdapter.invoke( {param($jsonInput) Get-ConfigObject -jsonInput $jsonInput}, $jsonInput )
        if ($null -eq $desiredState) {
            $trace = @{'Debug' = 'ERROR: Failed to create configuration object from provided JSON input.' } | ConvertTo-Json -Compress
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
        
        $resourceCache = Invoke-DscCacheRefresh -module $dscResourceModules
        if ($resourceCache.count -ne $dscResourceModules.count) {
            $trace = @{'Debug' = 'ERROR: DSC resource module not found.' } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
            exit 1
        }

        foreach ($ds in $desiredState) {
            # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
            $actualState = $psDscAdapter.invoke( {param($ds, $resourcecache) Get-ActualState -DesiredState $ds -ResourceCache $resourcecache}, $ds, $resourceCache)
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