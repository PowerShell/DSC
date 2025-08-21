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
}

Describe 'Tests for PowerShell resource discovery' {
    It 'Should find DSC PowerShell resources' {
        $out = dsc resource list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.directory | Should -Contain $TestDrive
    }
}
