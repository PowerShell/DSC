# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Assertion resource tests' {
    It 'Example works for <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
# TODO: Add export to test when https://github.com/PowerShell/DSC/issues/428 is fixed
#        @{ operation = 'export' }
    ) {
        param($operation)
        $jsonPath = Join-Path $PSScriptRoot '../examples/assertion.dsc.yaml'
        $out = dsc config $operation -f $jsonPath 2> "$TestDrive/trace.log"
        $LASTEXITCODE | Should -Be 2
        $out | Should -BeNullOrEmpty
        $log = Get-Content "$TestDrive/trace.log" -Raw
        $log | Should -Match '.*Assertion failed.*'
    }
}
