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
        if ($IsWindows) {
            $out.Count | Should -Be 2 -Because ($out | Out-String)
            $out[0].type | Should -Be 'Microsoft.Windows.Appx/Discover'
            $out[0].version | Should -Be '0.1.0'
            $out[0].capabilities | Should -BeExactly @('discover')
            $out[0].manifest | Should -Not -BeNullOrEmpty
            $out[1].type | Should -BeExactly 'Test/Discover'
            $out[1].version | Should -BeExactly '0.1.0'
            $out[1].capabilities | Should -BeExactly @('discover')
            $out[1].manifest | Should -Not -BeNullOrEmpty
        } else {
            $out.Count | Should -Be 1 -Because ($out | Out-String)
            $out[0].type | Should -BeExactly 'Test/Discover'
            $out[0].version | Should -BeExactly '0.1.0'
            $out[0].capabilities | Should -BeExactly @('discover')
            $out[0].manifest | Should -Not -BeNullOrEmpty
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

    It 'Failed extension discovery should not fail overall discovery' -Skip:(!$IsWindows) {
        try {
            # exclude finding powershell.exe
            $oldPath = $env:PATH
            $dscFolder = Split-Path (Get-Command dsc).Source -Parent
            $env:PATH = "$env:PROGRAMFILES\PowerShell\7;$dscFolder"
            $out = dsc -l warn resource list 2> $TestDrive/error.log | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.Count | Should -BeGreaterThan 0
            (Get-Content -Path "$TestDrive/error.log" -Raw) | Should -BeLike "*WARN Extension 'Microsoft.Windows.Appx/Discover' failed to discover resources: Command: Operation Executable 'powershell' not found*" -Because (Get-Content -Path "$TestDrive/error.log" -Raw | Out-String)
        } finally {
            $env:PATH = $oldPath
        }
    }

    It 'Deprecated extension shows message' {
        try {
            $dscHome = Split-Path (Get-Command dsc).Source -Parent
            $env:DSC_RESOURCE_PATH = (Join-Path -Path $dscHome -ChildPath 'deprecated') + [System.IO.Path]::PathSeparator + $dscHome

            $null = dsc resource list 2> $TestDrive/error.log
            $LASTEXITCODE | Should -Be 0
            (Get-Content -Path "$TestDrive/error.log" -Raw) | Should -Match "Extension 'Test/ExtensionDeprecated' is deprecated: This extension is deprecated" -Because (Get-Content -Path "$TestDrive/error.log" -Raw | Out-String)
        } finally {
            $env:DSC_RESOURCE_PATH = $null
        }
    }
}
