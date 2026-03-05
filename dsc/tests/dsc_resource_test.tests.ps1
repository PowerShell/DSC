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

    It 'stateAndDiff returns correctly when in desired state' {
        $out = '{"valueOne":1,"valueTwo":2}' | dsc resource test -r Test/StateAndDiff -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.valueOne | Should -Be 1
        $out.actualState.valueTwo | Should -Be 2
        $out.actualState._inDesiredState | Should -Be $true
        $out.inDesiredState | Should -Be $true
        $out.differingProperties | Should -BeNullOrEmpty
    }

    It 'stateAndDiff returns correctly when not in desired state' {
        $out = '{"valueOne":3,"valueTwo":4}' | dsc resource test -r Test/StateAndDiff -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.valueOne | Should -Be 1
        $out.actualState.valueTwo | Should -Be 2
        $out.actualState._inDesiredState | Should -Be $false
        $out.inDesiredState | Should -Be $false
        $out.differingProperties | Should -Contain 'valueOne'
        $out.differingProperties | Should -Contain 'valueTwo'
    }

    It 'stateAndDiff returns correctly when partially in desired state' {
        $out = '{"valueOne":1,"valueTwo":4}' | dsc resource test -r Test/StateAndDiff -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.valueOne | Should -Be 1
        $out.actualState.valueTwo | Should -Be 2
        $out.actualState._inDesiredState | Should -Be $false
        $out.inDesiredState | Should -Be $false
        $out.differingProperties | Should -Contain 'valueTwo'
        $out.differingProperties | Should -Not -Contain 'valueOne'
    }
}
