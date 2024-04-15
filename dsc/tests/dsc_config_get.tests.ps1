# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc config get tests' {
    It 'can successfully get config with multiple registry resource instances: <config>' -Skip:(!$IsWindows) -TestCases @(
        @{ config = 'osinfo_registry.dsc.json' }
        @{ config = 'osinfo_registry.dsc.yaml' }
    ) {
        param($config)
        $jsonPath = Join-Path $PSScriptRoot '../examples' $config
        $config = Get-Content $jsonPath -Raw
        $out = $config | dsc config get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.hadErrors | Should -BeFalse
        $out.results.Count | Should -Be 3
        $out.results[0].Name | Should -Be 'os'
        $out.results[0].type | Should -BeExactly 'Microsoft/OSInfo'
        $out.results[0].result.actualState.family | Should -BeExactly 'Windows'
        $out.results[1].Name | Should -Be 'windows product name'
        $out.results[1].type | Should -BeExactly 'Microsoft.Windows/Registry'
        $out.results[1].result.actualState.valueData.String | Should -BeLike 'Windows*'
        $out.results[2].Name | Should -Be 'system root'
        $out.results[2].type | Should -BeExactly 'Microsoft.Windows/Registry'
        $out.results[2].result.actualState.valueData.String | Should -BeExactly $env:SystemRoot
    }

    It 'will fail if resource schema does not match' -Skip:(!$IsWindows) {
        $jsonPath = Join-Path $PSScriptRoot '../examples/invalid_schema.dsc.yaml'
        $config = Get-Content $jsonPath -Raw
        $testError = & {$config | dsc config get get 2>&1}
        $testError[0] | Should -match 'error:'
        $LASTEXITCODE | Should -Be 2
    }

    It 'can accept the use of --format as a subcommand' {
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                output: hello
"@
        $result = $config_yaml | dsc config get --format pretty-json | ConvertFrom-Json
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $result.results[0].Name | Should -Be 'Echo'
        $result.results[0].type | Should -BeExactly 'Test/Echo'
        $result.results[0].result.actualState.output | Should -Be 'hello'
        $result.metadata.'Microsoft.DSC'.version | Should -BeLike '3.*'
        $result.metadata.'Microsoft.DSC'.operation | Should -BeExactly 'Get'
        $result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'Actual'
        $result.metadata.'Microsoft.DSC'.startDatetime | Should -Not -BeNullOrEmpty
        $result.metadata.'Microsoft.DSC'.endDatetime | Should -Not -BeNullOrEmpty
        $result.metadata.'Microsoft.DSC'.duration | Should -Not -BeNullOrEmpty
        $result.metadata.'Microsoft.DSC'.securityContext | Should -Not -BeNullOrEmpty
        $LASTEXITCODE | Should -Be 0
    }
}
