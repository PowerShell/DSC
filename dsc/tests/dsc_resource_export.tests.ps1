# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Resource export tests' {
    It "Export with <resource> accepts input '<json>' and returns filtered results" -TestCases @(
        @{ resource = 'Test/ExportSchemaCommand'; json = '{ "name": "Gijs" }'; expected = @('Gijs') }
        @{ resource = 'Test/ExportSchemaCommand'; json = '{ "name": "*e*" }'; expected = @('Steve', 'Tess') }
        @{ resource = 'Test/ExportSchemaEmbedded'; json = '{ "name": "Gijs" }'; expected = @('Gijs') }
        @{ resource = 'Test/ExportSchemaEmbedded'; json = '{ "name": "*e*" }'; expected = @('Steve', 'Tess') }
    ){
        param($resource, $json, $expected)

        $output = dsc resource export -r $resource -i $json  2>$TESTDRIVE/error.log | ConvertFrom-Json
        $errorlog = Get-Content "$TESTDRIVE/error.log" -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorlog
        $output.resources.count | Should -Be $expected.Count -Because ($output | ConvertTo-Json -Depth 4)
        $output.resources.properties.name | Should -Be $expected -Because ($output | ConvertTo-Json -Depth 4)
    }

    It "Engine filters input '<json>' for a resource without native filtering support" -TestCases @(
        @{ json = '{ "name": "Gijs" }'; expected = @('Gijs') }
        @{ json = '{ "name": "*e*" }'; expected = @('Steve', 'Tess') }
        @{ json = '[{ "name": "Gijs" }, { "name": "Steve" }]'; expected = @('Steve', 'Gijs') }
    ){
        param($json, $expected)

        $resource = 'Test/ExportSchemaNoFiltering'
        $output = dsc --trace-level info resource export -r $resource -i $json 2>$TESTDRIVE/error.log | ConvertFrom-Json
        $errorlog = Get-Content "$TESTDRIVE/error.log" -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorlog
        $output.resources.count | Should -Be $expected.Count -Because ($output | ConvertTo-Json -Depth 4)
        $output.resources.properties.name | Should -Be $expected -Because ($output | ConvertTo-Json -Depth 4)
        $errorlog | Should -Match "Resource '$resource' does not support export filtering, the engine will filter the exported instances \(experimental feature\)"
    }

    It 'Engine filtering rejects input that is not an object or array of objects' {
        $resource = 'Test/ExportSchemaNoFiltering'
        $json = '5'

        dsc resource export -r $resource -i $json 2>$TESTDRIVE/error.log | Out-Null
        $errorlog = Get-Content "$TESTDRIVE/error.log" -Raw
        $LASTEXITCODE | Should -Be 2 -Because $errorlog
        $errorlog | Should -Match 'Export filter input must be a JSON object or an array of JSON objects'
    }
}
