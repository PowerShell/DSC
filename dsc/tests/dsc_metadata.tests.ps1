# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'metadata tests' {
    It 'resource can provide metadata for <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                hello: world
                myNumber: 42
'@

        $out = dsc config $operation -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.hello | Should -BeExactly 'world'
        $out.results[0].metadata.myNumber | Should -Be 42
    }

    It 'resource can provide metadata for export' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                hello: There
                myNumber: 16
'@
        $out = dsc config export -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 3
        $out.resources[0].metadata.hello | Should -BeExactly 'There'
        $out.resources[0].metadata.myNumber | Should -Be 16
        $out.resources[0].name | Should -BeExactly 'Metadata example 1'
        $out.resources[1].metadata.hello | Should -BeExactly 'There'
        $out.resources[1].metadata.myNumber | Should -Be 16
        $out.resources[1].name | Should -BeExactly 'Metadata example 2'
        $out.resources[2].metadata.hello | Should -BeExactly 'There'
        $out.resources[2].metadata.myNumber | Should -Be 16
        $out.resources[2].name | Should -BeExactly 'Metadata example 3'
    }

    It 'resource returning Microsoft.DSC metadata is ignored' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                Microsoft.DSC:
                  hello: world
                validOne: true
'@
        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.validOne | Should -BeTrue
        $out.results[0].metadata.Microsoft.DSC | Should -BeNullOrEmpty
        (Get-Content $TestDrive/error.log) | Should -BeLike "*WARN*Resource returned metadata property 'Microsoft.DSC' which is ignored*"
    }
}