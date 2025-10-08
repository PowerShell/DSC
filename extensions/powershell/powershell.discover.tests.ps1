# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeAll {
    $fakeManifest = @{
        '$schema' = "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json"
        type = "Test/FakeResource"
        version = "0.1.0"
        get = @{
            executable = "fakeResource"
            args = @(
                "get",
                @{
                    jsonInputArg = "--input"
                    mandatory = $true
                }
            )
        }
    }
    
    $manifestPath = Join-Path $TestDrive "fake.dsc.resource.json"
    $fakeManifest | ConvertTo-Json -Depth 10 | Set-Content -Path $manifestPath
    $env:PSModulePath += [System.IO.Path]::PathSeparator + $TestDrive
    
    $script:discoverScript = Join-Path $PSScriptRoot "powershell.discover.ps1"
    
    $cacheFilePath = if ($IsWindows) {
        Join-Path $env:LocalAppData "dsc\PowerShellDiscoverCache.json"
    } else {
        Join-Path $env:HOME ".dsc" "PowerShellDiscoverCache.json"
    }
    $script:cacheFilePath = $cacheFilePath
}

Describe 'Tests for PowerShell resource discovery' {
    BeforeEach {
        # Clean cache before each test
        Remove-Item -Force -ErrorAction SilentlyContinue -Path $script:cacheFilePath
    }
    
    It 'Should find DSC PowerShell resources' {
        $out = dsc resource list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.directory | Should -Contain $TestDrive
    }
    
    It 'Should create cache file on first run' {
        $script:cacheFilePath | Should -Not -Exist
        
        $out = & $script:discoverScript 2>&1
        
        $script:cacheFilePath | Should -Exist
        
        $cache = Get-Content -Raw $script:cacheFilePath | ConvertFrom-Json
        $cache.PSModulePaths | Should -Not -BeNullOrEmpty
        $cache.PathInfo | Should -Not -BeNullOrEmpty
        $cache.Manifests | Should -Not -BeNullOrEmpty
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
}
