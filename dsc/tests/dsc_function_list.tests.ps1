# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for function list subcommand' {
    It 'Should list all available functions' {
        $out = dsc function list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.count | Should -BeGreaterThan 10
        $out.name | Should -Contain 'reference'
        $out.name | Should -Contain 'envvar'
        $out.description | Should -Not -Contain ''
        $out.category | Should -Not -Contain ''
    }

    It 'Should filter with wildcard' {
        $out = dsc function list 'resource*' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.category | Should -BeExactly 'Resource'
        $out.name | Should -BeExactly 'resourceId'
        $out.minArgs | Should -Be 2
        $out.maxArgs | Should -Be 2
        $out.acceptedArgTypes | Should -Be @('String')
        $out.description | Should -Not -BeNullOrEmpty
    }
}
