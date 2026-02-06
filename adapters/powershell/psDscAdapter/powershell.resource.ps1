# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate, ClearCache.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate', 'ClearCache')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '{}',
    [Parameter()]
    [string]$ResourceType
)

$traceQueue = [System.Collections.Concurrent.ConcurrentQueue[object]]::new()

function Write-DscTrace {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Operation,
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message,
        [switch]$Now
    )

    $trace = @{$Operation.ToLower() = $Message }

    if ($Now) {
        $host.ui.WriteErrorLine(($trace | ConvertTo-Json -Compress -Depth 10))
    } else {
        $traceQueue.Enqueue($trace)
    }
}

trap {
    Write-DscTrace -Operation Error -Message ($_ | Format-List -Force | Out-String)
}

function Write-TraceQueue() {
    $trace = $null
    while (!$traceQueue.IsEmpty) {
        if ($traceQueue.TryDequeue([ref] $trace)) {
            $host.ui.WriteErrorLine(($trace | ConvertTo-Json -Compress -Depth 10))
        }
    }
}

$ps = [PowerShell]::Create().AddScript({
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate, ClearCache.')]
        [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate', 'ClearCache')]
        [string]$Operation,
        [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
        [string]$jsonInput = '{}',
        [Parameter()]
        [string]$ResourceType,
        [Parameter()]
        [string]$ScriptRoot
    )

    trap {
        Write-Error ($_ | Format-List -Force | Out-String)
    }

    $DebugPreference = 'Continue'
    $VerbosePreference = 'Continue'
    $ErrorActionPreference = 'Continue'
    $InformationPreference = 'Continue'
    $ProgressPreference = 'SilentlyContinue'

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

        Remove-Item -Force -ErrorAction Ignore -Path $cacheFilePath
        exit
    }

    # Adding some debug info to STDERR
    Write-Debug ('PSVersion=' + $PSVersionTable.PSVersion.ToString())
    Write-Debug ('PSPath=' + $PSHome)
    Write-Debug ('PSModulePath=' + $env:PSModulePath)

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
        Write-Debug ("jsonInput=$jsonInput")

        # load private functions of psDscAdapter stub module
        if ($PSVersionTable.PSVersion.Major -le 5) {
            $psDscAdapter = Import-Module "$ScriptRoot/win_psDscAdapter.psd1" -Force -PassThru
        }
        else {
            $psDscAdapter = Import-Module "$ScriptRoot/psDscAdapter.psd1" -Force -PassThru
        }

        # initialize OUTPUT as array
        $result = [System.Collections.Generic.List[Object]]::new()
    }

    if ($jsonInput) {
        if ($jsonInput -ne '{}') {
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
                    if ($ResourceType) {
                        $requireAdapter = 'Microsoft.Adapter/WindowsPowerShell'
                    } else {
                        $requireAdapter = 'Microsoft.Windows/WindowsPowerShell'
                    }
                }
                else {
                    if ($ResourceType) {
                        $requireAdapter = 'Microsoft.Adapter/PowerShell'
                    } else {
                        $requireAdapter = 'Microsoft.DSC/PowerShell'
                    }
                }

                $properties = @()
                foreach ($prop in $DscResourceInfo.Properties) {
                    $properties += $prop.Name
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
                    properties     = $properties
                    requireAdapter = $requireAdapter
                    description    = $description
                }
            }
        }
        { @('Get','Set','Test','Export') -contains $_ } {
            if ($ResourceType) {
                Write-Debug ("Using resource type override: $ResourceType")
                $dscResourceCache = Invoke-DscCacheRefresh -Module $ResourceType.Split('/')[0]
                if ($null -eq $dscResourceCache) {
                    Write-Error ("DSC resource '{0}' module not found." -f $ResourceType)
                    exit 1
                }

                $desiredState = $psDscAdapter.invoke(   { param($jsonInput, $type) Get-DscResourceObject -jsonInput $jsonInput -type $type }, $jsonInput, $ResourceType )
                if ($null -eq $desiredState) {
                    Write-Error 'Failed to create configuration object from provided input JSON.'
                    exit 1
                }

                $desiredState.Type = $ResourceType
                $inDesiredState = $true
                $actualState = $psDscAdapter.invoke( { param($op, $ds, $dscResourceCache) Invoke-DscOperation -Operation $op -DesiredState $ds -dscResourceCache $dscResourceCache }, $Operation, $desiredState, $dscResourceCache)
                if ($null -eq $actualState) {
                    Write-Error 'Incomplete GET for resource ' + $desiredState.Name
                    exit 1
                }
                if ($actualState.Properties.InDesiredState -eq $false) {
                    $inDesiredState = $false
                }

                if ($Operation -in @('Set', 'Test')) {
                    $actualState = $psDscAdapter.Invoke( { param($ds, $dscResourceCache) Invoke-DscOperation -Operation 'Get' -DesiredState $ds -dscResourceCache $dscResourceCache }, $desiredState, $dscResourceCache)
                }

                if ($Operation -eq 'Test') {
                    $actualState.Properties | Add-Member -MemberType NoteProperty -Name _inDesiredState -Value $inDesiredState -Force
                }

                if ($Operation -eq 'Export') {
                    foreach ($instance in $actualState) {
                        $instance
                    }
                    exit 0
                }

                $result = $actualState.Properties
                return $result
            }

            $desiredState = $psDscAdapter.invoke(   { param($jsonInput) Get-DscResourceObject -jsonInput $jsonInput }, $jsonInput )
            if ($null -eq $desiredState) {
                Write-Error 'Failed to create configuration object from provided input JSON.'
                exit 1
            }

            # only need to cache the resources that are used
            $dscResourceModules = $desiredState | ForEach-Object { $_.Type.Split('/')[0] }
            if ($null -eq $dscResourceModules) {
                Write-Error 'Could not get list of DSC resource types from provided JSON.'
                exit 1
            }

            # get unique module names from the desiredState input
            $moduleInput = $desiredState | Select-Object -ExpandProperty Type | Sort-Object -Unique

            # refresh the cache with the modules that are available on the system
            $dscResourceCache = Invoke-DscCacheRefresh -module $dscResourceModules

            # check if all the desired modules are in the cache
            $moduleInput | ForEach-Object {
                if ($dscResourceCache.type -notcontains $_) {
                    Write-Error ("DSC resource '{0}' module not found." -f $_)
                    exit 1
                }
            }

            $inDesiredState = $true
            foreach ($ds in $desiredState) {
                # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
                $actualState = $psDscAdapter.invoke( { param($op, $ds, $dscResourceCache) Invoke-DscOperation -Operation $op -DesiredState $ds -dscResourceCache $dscResourceCache }, $Operation, $ds, $dscResourceCache)
                if ($null -eq $actualState) {
                    Write-Error ("Failed to invoke operation '{0}' for resource name '{1}'." -f $Operation, $ds.Name)
                    exit 1
                }
                if ($null -ne $actualState.Properties -and $actualState.Properties.InDesiredState -eq $false) {
                    $inDesiredState = $false
                }
                $result += $actualState
            }

            # OUTPUT json to stderr for debug, and to stdout
            if ($Operation -eq 'Test') {
                $result = @{ result = $result; _inDesiredState = $inDesiredState }
            }
            else {
                $result = @{ result = $result }
            }
            return $result
        }
        'Validate' {
            # VALIDATE not implemented

            # OUTPUT
            @{ valid = $true }
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
}).AddParameter('Operation', $Operation).AddParameter('jsonInput', $jsonInput).AddParameter('ResourceType', $ResourceType).AddParameter('ScriptRoot', $PSScriptRoot)

enum DscTraceLevel {
    Error
    Warn
    Info
    Debug
    Trace
}

$traceLevel = if ($env:DSC_TRACE_LEVEL) {
    try {
        [DscTraceLevel]$env:DSC_TRACE_LEVEL
    } catch {
        [DscTraceLevel]::Warn
    }
} else {
    [DscTraceLevel]::Warn
}

$null = Register-ObjectEvent -InputObject $ps.Streams.Error -EventName DataAdding -MessageData $traceQueue -Action {
    $traceQueue = $Event.MessageData
    # convert error to string since it's an ErrorRecord
    $traceQueue.Enqueue(@{ error = [string]$EventArgs.ItemAdded })
}

if ($traceLevel -ge [DscTraceLevel]::Warn) {
    $null = Register-ObjectEvent -InputObject $ps.Streams.Warning -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        $traceQueue.Enqueue(@{ warn = $EventArgs.ItemAdded.Message })
    }
}

if ($traceLevel -ge [DscTraceLevel]::Info) {
    $null = Register-ObjectEvent -InputObject $ps.Streams.Information -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        if ($null -ne $EventArgs.ItemAdded.MessageData) {
            if ($EventArgs.ItemAdded.Tags -contains 'PSHOST') {
                $traceQueue.Enqueue(@{ info = $EventArgs.ItemAdded.MessageData.ToString() })
            } else {
                $traceQueue.Enqueue(@{ trace = $EventArgs.ItemAdded.MessageData.ToString() })
            }
            return
        }
    }
}

if ($traceLevel -ge [DscTraceLevel]::Debug) {
    $null = Register-ObjectEvent -InputObject $ps.Streams.Verbose -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        # Verbose messages tend to be in large quantity and more useful to developers, so log as Debug
        $traceQueue.Enqueue(@{ debug = $EventArgs.ItemAdded.Message })
    }
}

if ($traceLevel -ge [DscTraceLevel]::Trace) {
    $null = Register-ObjectEvent -InputObject $ps.Streams.Debug -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        # Debug messages may contain raw info, so log as Trace
        $traceQueue.Enqueue(@{ trace = $EventArgs.ItemAdded.Message })
    }
}
$outputObjects = [System.Collections.Generic.List[Object]]::new()

try {
    $asyncResult = $ps.BeginInvoke()
    while (-not $asyncResult.IsCompleted) {
        Write-TraceQueue
    
        Start-Sleep -Milliseconds 100
    }
    $outputCollection = $ps.EndInvoke($asyncResult)
    Write-TraceQueue

    if ($ps.HadErrors) {
        # Anything written to stderr sets this flag, so we'll write a debug trace, but not treat as error
        Write-DscTrace -Now -Operation Debug -Message 'HadErrors set during script execution.'
    }

    foreach ($output in $outputCollection) {
        $outputObjects.Add($output)
    }
}
catch {
    Write-DscTrace -Now -Operation Error -Message $_
    exit 1
}
finally {
    $ps.Dispose()
    Get-EventSubscriber | Unregister-Event
}

foreach ($obj in $outputObjects) {
    $obj | ConvertTo-Json -Depth 10 -Compress
}
