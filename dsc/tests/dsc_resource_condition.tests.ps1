# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $configYaml = @'
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
        - name: test
        type: Microsoft.DSC.Debug/Echo
        condition: "[equals('skip', 'yes')]"
        properties:
            output: "This should not be executed"
        - name: test2
        type: Microsoft.DSC.Debug/Echo
        condition: "[equals('no', 'no')]"
        properties:
            output: "This should be executed"
'@

}

Describe 'Resource condition tests' {
    It 'resource should be skipped for <operation>' -TestCases @(
        @{ operation = 'get'; property = 'actualState' },
        @{ operation = 'set'; property = 'afterState' },
        @{ operation = 'test'; property = 'actualState' }
    ) {
        param($operation, $property)
        $out = dsc config $operation -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].result.$property.Output | Should -BeExactly "This should be executed"
    }

    It 'resource should be skipped for export' {
        $out = dsc config export -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 1
        $out.resources[0].type | Should -BeExactly 'Microsoft.DSC.Debug/Echo'
        $out.resources[0].properties.output | Should -BeExactly "This should be executed"
    }
}
