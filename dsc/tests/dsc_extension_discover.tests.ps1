# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    try {
        $windowWidth = [Console]::WindowWidth
    } catch {
        $consoleUnavailable = $true
    }
}

Describe 'Discover extension tests' {
    BeforeAll {
        $oldPath = $env:PATH
        $toolPath = Resolve-Path -Path "$PSScriptRoot/../../extensions/test/discover"
        $env:PATH = "$toolPath" + [System.IO.Path]::PathSeparator + $oldPath
    }

    AfterAll {
        $env:PATH = $oldPath
    }

    It 'Discover extensions' {
        $out = dsc extension list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $expectedExtensions = if ($IsWindows) {
            @(
            @{ type = 'Microsoft.DSC.Extension/Bicep'; version = '0.1.0'; capabilities = @('import') }
            @{ type = 'Microsoft.Windows.Appx/Discover'; version = '0.1.0'; capabilities = @('discover') }
            @{ type = 'Microsoft.PowerShell/Discover'; version = '0.1.0'; capabilities = @('discover') }
            @{ type = 'Test/Discover'; version = '0.1.0'; capabilities = @('discover') }
            )
        } else {
            @(
            @{ type = 'Microsoft.DSC.Extension/Bicep'; version = '0.1.0'; capabilities = @('import') }
            @{ type = 'Microsoft.PowerShell/Discover'; version = '0.1.0'; capabilities = @('discover') }
            @{ type = 'Test/Discover'; version = '0.1.0'; capabilities = @('discover') }
            )
        }

        $out.Count | Should -Be $expectedExtensions.Count -Because ($out | Out-String)
        
        foreach ($expected in $expectedExtensions) {
            $extension = $out | Where-Object { $_.type -eq $expected.type }
            $extension | Should -Not -BeNullOrEmpty -Because "Extension $($expected.type) should exist"
            $extension.version | Should -BeExactly $expected.version
            $extension.capabilities | Should -BeExactly $expected.capabilities
            $extension.manifest | Should -Not -BeNullOrEmpty
        }
    }

    It 'Filtering works for extension discovered resources' {
        $out = dsc resource list '*Discovered*' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.Count | Should -Be 2
        $out[0].type | Should -Be 'Test/DiscoveredOne'
        $out[1].type | Should -Be 'Test/DiscoveredTwo'
        $out[0].kind | Should -Be 'Resource'
        $out[1].kind | Should -Be 'Resource'
    }

    It 'Extension resources can be used in config' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            contentVersion: 1.0.0.0
            resources:
            - name: One
              type: Test/DiscoveredOne
            - name: Two
              type: Test/DiscoveredTwo
"@
        $out = dsc config get -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.Count | Should -Be 2
        $out.results[0].type | Should -BeExactly 'Test/DiscoveredOne'
        $out.results[0].result.actualState.Output | Should -BeExactly 'Hello One'
        $out.results[1].type | Should -BeExactly 'Test/DiscoveredTwo'
        $out.results[1].result.actualState.Output | Should -BeExactly 'Hello Two'
    }

    It 'Relative path from discovery will fail' {
        $extension_json = @'
{
    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
    "type": "Test/DiscoverRelative",
    "version": "0.1.0",
    "description": "Test discover resource",
    "discover": {
        "executable": "pwsh",
        "args": [
            "-NoLogo",
            "-NonInteractive",
            "-NoProfile",
            "-Command",
            "./discover.ps1",
            "-RelativePath"
            ]
    }
}
'@
        Set-Content -Path "$TestDrive/test.dsc.extension.json" -Value $extension_json
        Copy-Item -Path "$toolPath/discover.ps1" -Destination $TestDrive | Out-Null
        Copy-Item -Path "$toolPath/resources" -Destination $TestDrive -Recurse | Out-Null
        $env:DSC_RESOURCE_PATH = "$TestDrive" + [System.IO.Path]::PathSeparator + (Split-Path (Get-Command pwsh).Source -Parent)
        try {
            $out = dsc extension list | ConvertFrom-Json
            $out.Count | Should -Be 1 -Because ($out | Out-String)
            $out.type | Should -Be 'Test/DiscoverRelative'
            $out = dsc resource list 2> $TestDrive/error.log
            $LASTEXITCODE | Should -Be 0
            $out | Should -BeNullOrEmpty
            $errorMessage = Get-Content -Path "$TestDrive/error.log" -Raw
            $errorMessage | Should -BeLike '*is not an absolute path*'
        } finally {
            $env:DSC_RESOURCE_PATH = $null
        }
    }

    It 'Table can be not truncated' -Skip:($consoleUnavailable) {
        $output = dsc extension list --output-format table-no-truncate
        $LASTEXITCODE | Should -Be 0
        $foundWideLine = $false
        foreach ($line in $output) {
            if ($line.Length -gt $windowWidth) {
                $foundWideLine = $true
            }
        }
        $foundWideLine | Should -BeTrue
    }
}
