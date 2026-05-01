# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeAll {
    $fakeManifest = @{
        '$schema' = "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json"
        type      = "Test/FakeResource"
        version   = "0.1.0"
        get       = @{
            executable = "fakeResource"
            args       = @(
                "get",
                @{
                    jsonInputArg = "--input"
                    mandatory    = $true
                }
            )
        }
    }
    
    $manifestPath = Join-Path $TestDrive "fake.dsc.resource.json"
    $fakeManifest | ConvertTo-Json -Depth 10 | Set-Content -Path $manifestPath

    $fakeAdaptedManifest = @{
        '$schema'      = "https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json"
        type           = "Test/FakeAdaptedResource"
        kind           = "resource"
        version        = "0.1.0"
        capabilities   = @("get", "set", "test")
        description    = "A fake adapted resource for regression testing."
        requireAdapter = "Microsoft.Adapter/PowerShell"
        path           = "FakeAdapted.psd1"
        schema         = @{
            embedded = @{
                '$schema'            = "https://json-schema.org/draft/2020-12/schema"
                title                = "Test/FakeAdaptedResource"
                type                 = "object"
                required             = @("Name")
                additionalProperties = $false
                properties           = @{
                    Name = @{ type = "string"; title = "Name"; description = "The name." }
                }
            }
        }
    }

    $adaptedManifestPath = Join-Path $TestDrive "fake.dsc.adaptedresource.json"
    $fakeAdaptedManifest | ConvertTo-Json -Depth 10 | Set-Content -Path $adaptedManifestPath

    $fakeManifestList = @{
        '$schema'        = "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json"
        resources        = @(
            @{
                type    = "Test/FakeListedResource"
                version = "0.1.0"
                kind    = "resource"
                get     = @{
                    executable = "fakeListedResource"
                    args       = @(
                        "get",
                        @{
                            jsonInputArg = "--input"
                            mandatory    = $true
                        }
                    )
                }
            }
        )
        adaptedResources = @(
            @{
                type           = "Test/FakeListedAdaptedResource"
                kind           = "resource"
                version        = "0.1.0"
                requireAdapter = "Microsoft.Adapter/PowerShell"
                path           = "FakeAdapted.psd1"
            }
        )
    }

    $manifestListPath = Join-Path $TestDrive "fake.dsc.manifests.json"
    $fakeManifestList | ConvertTo-Json -Depth 10 | Set-Content -Path $manifestListPath

    $fakePsd1Path = Join-Path $TestDrive "FakeAdapted.psd1"
    Set-Content -Path $fakePsd1Path -Value "@{ ModuleVersion = '0.1.0' }"
    $script:OldPSModulePath = $env:PSModulePath
    $env:PSModulePath += [System.IO.Path]::PathSeparator + $TestDrive
    
    $script:discoverScript = Join-Path $PSScriptRoot "powershell.discover.ps1"
    
    $cacheFilePath = if ($IsWindows) {
        Join-Path $env:LocalAppData "dsc\PowerShellDiscoverCache.json"
    } else {
        Join-Path $env:HOME ".dsc" "PowerShellDiscoverCache.json"
    }
    $script:cacheFilePath = $cacheFilePath

    Remove-Item -Force -ErrorAction SilentlyContinue -Path $script:cacheFilePath
}

AfterAll {
    $env:PSModulePath = $script:OldPSModulePath
}

Describe 'Tests for PowerShell resource discovery' {    
    It 'Should create cache file on first run' {
        $script:cacheFilePath | Should -Not -Exist
        $null = & $script:discoverScript 2>&1
        $script:cacheFilePath | Should -Exist
        
        $cache = Get-Content -Raw $script:cacheFilePath | ConvertFrom-Json
        $cache.PSModulePaths | Should -Not -BeNullOrEmpty
        $cache.PathInfo | Should -Not -BeNullOrEmpty
        $cache.Manifests | Should -Not -BeNullOrEmpty
    }

    It 'Should find DSC PowerShell resources' {
        $out = dsc resource list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.manifest.type | Should -Contain 'Test/FakeResource'
    }
    
    It 'Should use cache on subsequent runs' {
        $null = & $script:discoverScript 2>&1
        $script:cacheFilePath | Should -Exist
        
        $cacheLastWriteTime = (Get-Item $script:cacheFilePath).LastWriteTimeUtc
        
        Start-Sleep -Milliseconds 100
        
        $null = & $script:discoverScript 2>&1
        
        $newLastWriteTime = (Get-Item $script:cacheFilePath).LastWriteTimeUtc
        $newLastWriteTime | Should -Be $cacheLastWriteTime
    }
    
    It 'Should invalidate cache when PSModulePath changes' {
        $null = & $script:discoverScript 2>&1
        $script:cacheFilePath | Should -Exist
        
        $cache = Get-Content -Raw $script:cacheFilePath | ConvertFrom-Json
        $originalPaths = $cache.PSModulePaths
        $cache.PSModulePaths = @($originalPaths[0])  # Remove some paths
        $cache | ConvertTo-Json -Depth 10 | Set-Content -Path $script:cacheFilePath -Force
        
        $cacheLastWriteTime = (Get-Item $script:cacheFilePath).LastWriteTimeUtc
        Start-Sleep -Milliseconds 100
        
        $null = & $script:discoverScript 2>&1
        
        $newLastWriteTime = (Get-Item $script:cacheFilePath).LastWriteTimeUtc
        $newLastWriteTime | Should -Not -Be $cacheLastWriteTime
    }
    
    It 'Should invalidate cache when module directory is modified' {
        $null = & $script:discoverScript 2>&1
        $script:cacheFilePath | Should -Exist
        
        $cache = Get-Content -Raw $script:cacheFilePath | ConvertFrom-Json
        
        $firstPath = $cache.PathInfo.PSObject.Properties | Select-Object -First 1
        if ($firstPath) {
            $oldTimestamp = [DateTime]$firstPath.Value
            $newTimestamp = $oldTimestamp.AddDays(-1)
            $cache.PathInfo.($firstPath.Name) = $newTimestamp
            $cache | ConvertTo-Json -Depth 10 | Set-Content -Path $script:cacheFilePath -Force
            
            $cacheLastWriteTime = (Get-Item $script:cacheFilePath).LastWriteTimeUtc
            Start-Sleep -Milliseconds 100
            
            $null = & $script:discoverScript 2>&1
            
            $newLastWriteTime = (Get-Item $script:cacheFilePath).LastWriteTimeUtc
            $newLastWriteTime | Should -Not -Be $cacheLastWriteTime
        }
    }
    
    It 'Should rebuild cache if cache file is corrupted' {
        "{ invalid json }" | Set-Content -Path $script:cacheFilePath -Force
        $script:cacheFilePath | Should -Exist
        
        $null = & $script:discoverScript 2>&1
        
        $cache = Get-Content -Raw $script:cacheFilePath | ConvertFrom-Json
        $cache.PSModulePaths | Should -Not -BeNullOrEmpty
        $cache.PathInfo | Should -Not -BeNullOrEmpty
    }
    
    It 'Should include test manifest in discovery results' {
        $out = & $script:discoverScript | ConvertFrom-Json
        $out.manifestPath | Should -Contain $manifestPath
    }

    It 'Should discover adapted resource manifest files' {
        Remove-Item -Force -ErrorAction SilentlyContinue -Path $script:cacheFilePath
        $out = & $script:discoverScript | ConvertFrom-Json
        $out.manifestPath | Should -Contain $adaptedManifestPath
    }

    It 'Should discover *.dsc.manifests.* manifest-list files' {
        Remove-Item -Force -ErrorAction SilentlyContinue -Path $script:cacheFilePath
        $out = & $script:discoverScript | ConvertFrom-Json
        $out.manifestPath | Should -Contain $manifestListPath
    }
}
