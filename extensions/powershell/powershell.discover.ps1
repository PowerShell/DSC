# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param ()

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

function Get-CacheFilePath {
    if ($IsWindows) {
        Join-Path $env:LocalAppData "dsc\PowerShellDiscoverCache.json"
    } else {
        Join-Path $env:HOME ".dsc" "PowerShellDiscoverCache.json"
    }
}

function Test-CacheValid {
    param([string]$CacheFilePath, [string[]]$PSPaths)
    
    if (-not (Test-Path $CacheFilePath)) {
        "Cache file not found '$CacheFilePath'" | Write-DscTrace
        return $false
    }
    
    try {
        "Reading cache file '$CacheFilePath'" | Write-DscTrace
        $cache = Get-Content -Raw $CacheFilePath | ConvertFrom-Json
        
        "Checking cache for stale entries" | Write-DscTrace
        foreach ($entry in $cache.PathInfo.PSObject.Properties) {
            $path = $entry.Name
            if (-not (Test-Path $path)) {
                "Detected non-existent cache entry '$path'" | Write-DscTrace
                return $false
            }
            
            $currentLastWrite = (Get-Item $path).LastWriteTimeUtc
            $cachedLastWrite = [DateTime]$entry.Value
            
            if ($currentLastWrite -ne $cachedLastWrite) {
                "Detected stale cache entry '$path' (cached: $cachedLastWrite, current: $currentLastWrite)" | Write-DscTrace
                return $false
            }
        }
        
        "Checking cache for stale PSModulePath" | Write-DscTrace
        $cachedPaths = [string[]]$cache.PSModulePaths
        if ($cachedPaths.Count -ne $PSPaths.Count) {
            "PSModulePath count changed (cached: $($cachedPaths.Count), current: $($PSPaths.Count))" | Write-DscTrace
            return $false
        }
        
        $diff = Compare-Object $cachedPaths $PSPaths
        if ($null -ne $diff) {
            "PSModulePath contents changed" | Write-DscTrace
            return $false
        }
        
        "Cache is valid" | Write-DscTrace
        return $true
    } catch {
        "Stale cached entries detected: $_" | Write-DscTrace
        return $false
    }
}

function Invoke-DscResourceDiscovery {
    [CmdletBinding()]
    param()
    
    begin {
        $psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }
        "Discovered $($psPaths.Count) PSModulePath segments (excluding WindowsPowerShell)" | Write-DscTrace
        
        $cacheFilePath = Get-CacheFilePath
        $useCache = Test-CacheValid -CacheFilePath $cacheFilePath -PSPaths $psPaths
    }
    process {
        if ($useCache) {
            "Using cached manifests" | Write-DscTrace
            $cache = Get-Content -Raw $cacheFilePath | ConvertFrom-Json
            $manifests = $cache.Manifests
            "Retrieved $($manifests.Count) manifests from cache" | Write-DscTrace
        } else {
            "Performing full discovery" | Write-DscTrace
            $manifests = $psPaths | ForEach-Object -Parallel {
                $searchPatterns = @('*.dsc.resource.json', '*.dsc.resource.yaml', '*.dsc.resource.yml')
                $enumOptions = [System.IO.EnumerationOptions]@{ IgnoreInaccessible = $false; RecurseSubdirectories = $true }
                foreach ($pattern in $searchPatterns) {
                    try {
                        [System.IO.Directory]::EnumerateFiles($_, $pattern, $enumOptions) | ForEach-Object {
                            @{ manifestPath = $_ }
                        }
                    } catch { }
                }
            } -ThrottleLimit 10
            
            "Discovered $($manifests.Count) manifests" | Write-DscTrace
            
            "Building cache" | Write-DscTrace
            $pathInfo = @{}
            foreach ($path in $psPaths) {
                if (Test-Path $path) {
                    $pathInfo[$path] = (Get-Item $path).LastWriteTimeUtc
                }
            }
            
            $cacheObject = @{
                PSModulePaths = $psPaths
                PathInfo = $pathInfo
                Manifests = $manifests
            }
            
            $cacheDir = Split-Path $cacheFilePath -Parent
            if (-not (Test-Path $cacheDir)) {
                "Creating cache directory '$cacheDir'" | Write-DscTrace
                New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null
            }
            "Saving cache to '$cacheFilePath'" | Write-DscTrace
            $cacheObject | ConvertTo-Json -Depth 10 | Set-Content -Path $cacheFilePath -Force
        }
    }
    end {
        $manifests | ForEach-Object { $_ | ConvertTo-Json -Compress }
    }
}

Invoke-DscResourceDiscovery

