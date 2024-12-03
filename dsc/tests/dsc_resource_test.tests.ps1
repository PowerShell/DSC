# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Invoke a resource test directly' {
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

    It 'test returns proper error code if no input is provded' {
        $out = dsc resource test -r Microsoft/OSInfo 2>&1
        $LASTEXITCODE | Should -Be 1
        $out | Should -BeLike '*ERROR*'
    }
}
