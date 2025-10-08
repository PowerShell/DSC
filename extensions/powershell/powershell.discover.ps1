# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param ()

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
        return $false
    }
    
    try {
        $cache = Get-Content -Raw $CacheFilePath | ConvertFrom-Json
        
        foreach ($entry in $cache.PathInfo.PSObject.Properties) {
            $path = $entry.Name
            if (-not (Test-Path $path)) {
                return $false
            }
            
            $currentLastWrite = (Get-Item $path).LastWriteTimeUtc
            $cachedLastWrite = [DateTime]$entry.Value
            
            if ($currentLastWrite -ne $cachedLastWrite) {
                return $false
            }
        }
        
        $cachedPaths = [string[]]$cache.PSModulePaths
        if ($cachedPaths.Count -ne $PSPaths.Count) {
            return $false
        }
        
        $diff = Compare-Object $cachedPaths $PSPaths
        if ($null -ne $diff) {
            return $false
        }
        
        return $true
    } catch {
        return $false
    }
}

function Invoke-DscResourceDiscovery {
    [CmdletBinding()]
    param()
    
    begin {
        $psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }
        
        $cacheFilePath = Get-CacheFilePath
        $useCache = Test-CacheValid -CacheFilePath $cacheFilePath -PSPaths $psPaths
    }
    process {
        if ($useCache) {
            $cache = Get-Content -Raw $cacheFilePath | ConvertFrom-Json
            $manifests = $cache.Manifests
        } else {
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
            
            $pathInfo = @{}
            foreach ($path in $psPaths) {
                if (Test-Path $path) {
                    $pathInfo[$path] = (Get-Item $path).LastWriteTimeUtc
                }
            }
            
            $cacheObject = @{
                PSModulePaths = $psPaths
                PathInfo      = $pathInfo
                Manifests     = $manifests
            }
            
            $cacheDir = Split-Path $cacheFilePath -Parent
            if (-not (Test-Path $cacheDir)) {
                New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null
            }

            $cacheObject | ConvertTo-Json -Depth 10 | Set-Content -Path $cacheFilePath -Force
        }
    }
    end {
        if ($null -eq $manifests -or [string]::IsNullOrEmpty($manifests)) {
            # Return nothing
        } else {
            $manifests | ForEach-Object { $_ | ConvertTo-Json -Compress }
        }
    }
}

Invoke-DscResourceDiscovery

