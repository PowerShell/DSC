# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate, ClearCache.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate', 'ClearCache')]
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

    $trace = @{$Operation.ToLower() = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

trap {
    Write-DscTrace -Operation Debug -Message ($_ | Format-List -Force | Out-String)
}

if ($Operation -eq 'ClearCache') {
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

    Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath
    exit 0
}

# Adding some debug info to STDERR
'PSVersion=' + $PSVersionTable.PSVersion.ToString() | Write-DscTrace
'PSPath=' + $PSHome | Write-DscTrace
'PSModulePath=' + $env:PSModulePath | Write-DscTrace

if ($PSVersionTable.PSVersion.Major -le 5) {
    # For Windows PowerShell, we want to remove any PowerShell 7 paths from PSModulePath
    if ($pwshPath = Get-Command 'pwsh' -ErrorAction Ignore | Select-Object -ExpandProperty Source) {
        $pwshDefaultModulePaths = @(
            "$HOME\Documents\PowerShell\Modules"                # CurrentUser
            "$Env:ProgramFiles\PowerShell\Modules"              # AllUsers
            Join-Path $(Split-Path $pwshPath -Parent) 'Modules' # Builtin
        )
        $env:PSModulePath = ($env:PSModulePath -split ';' | Where-Object { $_ -notin $pwshDefaultModulePaths }) -join ';'
    }
}

if ('Validate' -ne $Operation) {
    Write-DscTrace -Operation Debug -Message "jsonInput=$jsonInput"

    # load private functions of psDscAdapter stub module
    if ($PSVersionTable.PSVersion.Major -le 5) {
        $psDscAdapter = Import-Module "$PSScriptRoot/win_psDscAdapter.psd1" -Force -PassThru
    }
    else {
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
    if ($new_psmodulepath)
    {
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
        $desiredState = $psDscAdapter.invoke(   { param($jsonInput) Get-DscResourceObject -jsonInput $jsonInput }, $jsonInput )
        if ($null -eq $desiredState) {
            Write-DscTrace -Operation Error -message 'Failed to create configuration object from provided input JSON.'
            exit 1
        }

        # only need to cache the resources that are used
        $dscResourceModules = $desiredState | ForEach-Object { $_.Type.Split('/')[0] }
        if ($null -eq $dscResourceModules) {
            Write-DscTrace -Operation Error -Message 'Could not get list of DSC resource types from provided JSON.'
            exit 1
        }

        # get unique module names from the desiredState input
        $moduleInput = $desiredState | Select-Object -ExpandProperty Type | Sort-Object -Unique

        # refresh the cache with the modules that are available on the system
        $dscResourceCache = Invoke-DscCacheRefresh -module $dscResourceModules

        # check if all the desired modules are in the cache
        $moduleInput | ForEach-Object {
            if ($dscResourceCache.type -notcontains $_) {
                ("DSC resource '{0}' module not found." -f $_) | Write-DscTrace -Operation Error
                exit 1
            }
        }

        $inDesiredState = $true
        foreach ($ds in $desiredState) {
            # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
            $actualState = $psDscAdapter.invoke( { param($op, $ds, $dscResourceCache) Invoke-DscOperation -Operation $op -DesiredState $ds -dscResourceCache $dscResourceCache }, $Operation, $ds, $dscResourceCache)
            if ($null -eq $actualState) {
                Write-DscTrace -Operation Error -Message 'Incomplete GET for resource ' + $ds.Name
                exit 1
            }
            if ($null -ne $actualState.Properties -and $actualState.Properties.InDesiredState -eq $false) {
                $inDesiredState = $false
            }
            $result += $actualState
        }

        # OUTPUT json to stderr for debug, and to stdout
        if ($Operation -eq 'Test') {
            $result = @{ result = $result; _inDesiredState = $inDesiredState } | ConvertTo-Json -Depth 10 -Compress
        }
        else {
            $result = @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
        }
        Write-DscTrace -Operation Debug -Message "jsonOutput=$result"
        return $result
    }
    'Validate' {
        # VALIDATE not implemented

        # OUTPUT
        @{ valid = $true } | ConvertTo-Json
    }
    Default {
        Write-DscTrace -Operation Error -Message 'Unsupported operation. Please use one of the following: List, Get, Set, Test, Export, Validate'
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
