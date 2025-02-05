# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for config using reference function' {
    It 'Reference works for <operation>' -TestCases @(
        @{ operation = 'get' },
        @{ operation = 'test' }
    ) {
        param($operation)

        $out = dsc config $operation -f $PSScriptRoot/../examples/reference.dsc.yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $os = if ($IsWindows) {
            'Windows'
        }
        elseif ($IsLinux) {
            'Linux'
        }
        else {
            'macOS'
        }

        $out.results[1].result.actualState.Output | Should -BeExactly "The OS is $os"
    }
}
