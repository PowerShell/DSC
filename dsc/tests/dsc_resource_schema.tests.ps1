# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for the Resource schema within a configuration' {
    It 'Unknown properties are an error' {
        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: test
              type: Microsft.Dsc.Debug/Echo
              unknownProperty: true
              properties:
              output: "Hello World"
'@
        dsc config get -i $yaml 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        (Get-Content $TestDrive/error.log -Raw) | Should -Match 'Error: JSON: unknown field `unknownProperty`'

    }

    It 'dsc schema returns a valid schema' {
        $schema = dsc schema -t resource
        $LASTEXITCODE | Should -Be 0
        $schema | Should -Not -BeNullOrEmpty
        $schema = $schema | ConvertFrom-Json
        $schema.'$schema' | Should -BeExactly 'https://json-schema.org/draft/2020-12/schema'
        $schema.title | Should -BeExactly 'Resource'
        $schema.additionalProperties | Should -Be $false
    }

    It 'adapted resource manifest can return schema' {
        $schema = dsc resource schema -r Adapted/Two | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $schema.'$id' | Should -BeExactly 'https://github.com/powershell/dsc'
        $schema.title | Should -BeExactly 'AdaptedTwo'
        $schema.description | Should -BeExactly 'Adapted test resource number two'
        $schema.type | Should -BeExactly 'object'
        $schema.additionalProperties | Should -Be $false
        $schema.properties.two.type | Should -BeExactly 'string'
        $schema.properties.name.title | Should -BeExactly 'Name'
    }

    It 'resource schema can be returned from adapter' {
        $schema = dsc resource schema -r Adapted/One | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $schema.title | Should -BeExactly 'AdaptedOne'
        $schema.type | Should -BeExactly 'object'
        $schema.properties.one.type | Should -BeExactly 'string'
        $schema.properties._name.type | Should -BeExactly @('string', 'null')
        $schema.properties.path.type | Should -BeExactly @('string', 'null')
        $schema.additionalProperties | Should -Be $false
        $schema.required | Should -BeExactly @('one')
    }
}
