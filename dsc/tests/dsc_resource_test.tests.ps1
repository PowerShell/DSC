# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Invoke a resource test directly' {
    BeforeAll {
        $env:DSC_TRACE_LEVEL = 'error'
    }

    AfterAll {
        $env:DSC_TRACE_LEVEL = $null
    }

    It 'test can be called on a resource' {
        $os = if ($IsWindows) {
            'Windows'
        } elseif ($IsLinux) {
            'Linux'
        } elseif ($IsMacOS) {
            'macOS'
        } else {
            'Unknown'
        }

        $out = @"
        { "family": "$os" }
"@ | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.family | Should -BeExactly $os
        $out.inDesiredState | Should -Be $true
    }

    It 'test returns proper error code if no input is provided' {
        $out = dsc resource test -r Microsoft/OSInfo 2>&1
        $LASTEXITCODE | Should -Be 1
        $out | Should -BeLike '*ERROR*'
    }

     It 'version works' {
        $out = dsc resource test -r Test/Version --version 1.1.2 --input '{"version":"1.1.2"}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.version | Should -BeExactly '1.1.2'
        $out.inDesiredState | Should -Be $true
    }
}
