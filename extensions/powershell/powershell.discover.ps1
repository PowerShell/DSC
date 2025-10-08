# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param ()

# begin {
#     $psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }
# }
# process {
#     $manifests = $psPaths | ForEach-Object -Parallel {
#         $searchPatterns = @('*.dsc.resource.json', '*.dsc.resource.yaml', '*.dsc.resource.yml')
#         $enumOptions = [System.IO.EnumerationOptions]@{ IgnoreInaccessible = $false; RecurseSubdirectories = $true }
#         foreach ($pattern in $searchPatterns) {
#             try {
#                 [System.IO.Directory]::EnumerateFiles($_, $pattern, $enumOptions) | ForEach-Object {
#                     @{ manifestPath = $_ }
#                 }
#             } catch { }
#         }
#     } -ThrottleLimit 10
# }
# end {
#     $manifests | ForEach-Object { $_ | ConvertTo-Json -Compress }
# }


# [CmdletBinding()]
# param ()

# function Write-DscTrace {
#     param(
#         [Parameter(Mandatory = $false)]
#         [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
#         [string]$Operation = 'Debug',

#         [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
#         [string]$Message
#     )

#     $trace = @{$Operation.ToLower() = $Message } | ConvertTo-Json -Compress
#     $host.ui.WriteErrorLine($trace)
# }

# function Get-CacheFilePath {
#     if ($IsWindows) {
#         Join-Path $env:LocalAppData "dsc\PowerShellDiscoverCache.json"
#     } else {
#         Join-Path $env:HOME ".dsc" "PowerShellDiscoverCache.json"
#     }
# }

# function Test-CacheValid {
#     param([string]$CacheFilePath, [string[]]$PSPaths)
    
#     if (-not (Test-Path $CacheFilePath)) {
#         "Cache file not found '$CacheFilePath'" | Write-DscTrace
#         return $false
#     }
    
#     try {
#         "Reading cache file '$CacheFilePath'" | Write-DscTrace
#         $cache = Get-Content -Raw $CacheFilePath | ConvertFrom-Json
        
#         "Checking cache for stale entries" | Write-DscTrace
#         foreach ($entry in $cache.PathInfo.PSObject.Properties) {
#             $path = $entry.Name
#             if (-not (Test-Path $path)) {
#                 "Detected non-existent cache entry '$path'" | Write-DscTrace
#                 return $false
#             }
            
#             $currentLastWrite = (Get-Item $path).LastWriteTimeUtc
#             $cachedLastWrite = [DateTime]$entry.Value
            
#             if ($currentLastWrite -ne $cachedLastWrite) {
#                 "Detected stale cache entry '$path' (cached: $cachedLastWrite, current: $currentLastWrite)" | Write-DscTrace
#                 return $false
#             }
#         }
        
#         "Checking cache for stale PSModulePath" | Write-DscTrace
#         $cachedPaths = [string[]]$cache.PSModulePaths
#         if ($cachedPaths.Count -ne $PSPaths.Count) {
#             "PSModulePath count changed (cached: $($cachedPaths.Count), current: $($PSPaths.Count))" | Write-DscTrace
#             return $false
#         }
        
#         $diff = Compare-Object $cachedPaths $PSPaths
#         if ($null -ne $diff) {
#             "PSModulePath contents changed" | Write-DscTrace
#             return $false
#         }
        
#         "Cache is valid" | Write-DscTrace
#         return $true
#     } catch {
#         "Stale cached entries detected: $_" | Write-DscTrace
#         return $false
#     }
# }

function Invoke-DscResourceDiscovery {
    [CmdletBinding()]
    param()
    
    begin {
        $psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }
    }
    process {
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
    }
    end {
        $manifests | ForEach-Object { $_ | ConvertTo-Json -Compress }
    }
}

Invoke-DscResourceDiscovery

