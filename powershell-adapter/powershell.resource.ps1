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
Import-Module "$PSScriptRoot/psDscAdapter/psDscAdapter.psd1" -Force

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

# format expected for configuration and resource output
class configFormat {
    [string] $name
    [string] $type
    [psobject] $properties
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