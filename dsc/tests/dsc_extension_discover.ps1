# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Discover extension tests' {
    BeforeAll {
        $oldPath = $env:PATH
        $separator = [System.IO.Path]::PathSeparator
        $env:PATH = "$PSScriptRoot$separator$oldPath"
    }

    AfterAll {
        $env:PATH = $oldPath
    }

    It 'Discover extensions' {
        $out = dsc extension list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.Count | Should -Be 1
        $out.type | Should -BeExactly 'Test/Discover'
        $out.version | Should -BeExactly '0.1.0'
        $out.capabilities | Should -BeExactly @('discover')
        $out.manifest | Should -Not -BeNullOrEmpty
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
}
