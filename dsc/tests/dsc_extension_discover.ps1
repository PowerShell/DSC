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
}
