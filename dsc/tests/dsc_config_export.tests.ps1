# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config export tests' {
    It 'Execution information is included in config export results' {
        $config_yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: os
  type: Microsoft/OSInfo
'@

        $out = dsc config export -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.executionInformation | Should -Not -BeNullOrEmpty
        $out.executionInformation.startDatetime | Should -Not -BeNullOrEmpty
        $out.executionInformation.endDatetime | Should -Not -BeNullOrEmpty
        $out.executionInformation.duration | Should -Not -BeNullOrEmpty
        $out.executionInformation.operation | Should -BeExactly 'export'
        $out.executionInformation.executionType | Should -BeExactly 'actual'
        $out.executionInformation.securityContext | Should -Not -BeNullOrEmpty
        $out.executionInformation.version | Should -BeExactly (dsc --version).replace("dsc ", "")
        $out.resources | Should -Not -BeNullOrEmpty
        $out.resources.count | Should -Be 1
        $out.resources[0].Name | Should -Not -BeNullOrEmpty
        $out.resources[0].type | Should -BeExactly 'Microsoft/OSInfo'
        $out.resources[0].executionInformation | Should -Not -BeNullOrEmpty
        $out.resources[0].executionInformation.duration | Should -Not -BeNullOrEmpty
        $out.resources[0].properties.family | Should -BeIn @('Windows', 'Linux', 'macOS')
        $out.resources[0].properties.architecture | Should -BeIn @('x86_64', 'arm64')
        $out.resources[0].properties.version | Should -Not -BeNullOrEmpty
        $out.resources[0].properties.bitness | Should -BeIn @(32, 64)
    }
}
