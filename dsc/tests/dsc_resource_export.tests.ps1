# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Resource export tests' {
    It "Export with <resource> accepts input '<json>' and returns filtered results" -TestCases @(
        @{ resource = 'Test/ExportSchemaCommand'; json = '{ "name": "Gijs" }'; expected = @('Gijs') },
        @{ resource = 'Test/ExportSchemaCommand'; json = '{ "name": "*e*" }'; expected = @('Steve', 'Tess') },
        @{ resource = 'Test/ExportSchemaEmbedded'; json = '{ "name": "Gijs" }'; expected = @('Gijs') },
        @{ resource = 'Test/ExportSchemaEmbedded'; json = '{ "name": "*e*" }'; expected = @('Steve', 'Tess') },
        @{ resource = 'Test/ExportSchemaNoFiltering'; json = '{ "name": "Gijs" }'; expected = @('Steve', 'Tess', 'Gijs') }
    ){
        param($resource, $json, $expected)

        $output = dsc resource export -r $resource -i $json  2>$TESTDRIVE/error.log | ConvertFrom-Json
        $errorlog = Get-Content "$TESTDRIVE/error.log" -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorlog
        $output.resources.count | Should -Be $expected.Count -Because ($output | ConvertTo-Json -Depth 4)
        $output.resources.properties.name | Should -Be $expected -Because ($output | ConvertTo-Json -Depth 4)
    }
}
