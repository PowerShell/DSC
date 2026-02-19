# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Deprecated resource tests' {
    It 'Deprecated resource for operation <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'delete' }
        @{ operation = 'test' }
        @{ operation = 'export' }
    ) {
        param($operation)

        $out = dsc resource $operation -r Test/OperationDeprecated -i '{}' 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
        if ($operation -eq 'delete') {
            $out | Should -BeNullOrEmpty
        } else {
            $out | Should -Not -BeNullOrEmpty
        }
        (Get-Content $TestDrive/error.log -Raw) | Should -Match "Resource 'Test/OperationDeprecated' is deprecated: This resource is deprecated"
    }

    It 'Deprecated resource for schema' {
        $out = dsc resource schema -r Test/OperationDeprecated 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
        $out | Should -Not -BeNullOrEmpty
        (Get-Content $TestDrive/error.log -Raw) | Should -Match "Resource 'Test/OperationDeprecated' is deprecated: This resource is deprecated"
    }

    It 'Deprecated message when used in config' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/OperationDeprecated
            properties:
              operation: get
'@

        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
        $out.results.count | Should -Be 1
        $out.results[0].type | Should -BeExactly 'Test/OperationDeprecated'
        (Get-Content $TestDrive/error.log -Raw) | Should -Match "Resource 'Test/OperationDeprecated' is deprecated: This resource is deprecated"
    }
}
