# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for resource versioning' {
    It "Should return the correct version '<version>' for operation '<operation>'" -TestCases @(
        @{ version = '1.1.2'; operation = 'get'; property = 'actualState' }
        @{ version = '1.1.0'; operation = 'get'; property = 'actualState' }
        @{ version = '2.0.0'; operation = 'get'; property = 'actualState' }
        @{ version = '1.1.2'; operation = 'set'; property = 'afterState' }
        @{ version = '1.1.0'; operation = 'set'; property = 'afterState' }
        @{ version = '2.0.0'; operation = 'set'; property = 'afterState' }
        @{ version = '1.1.2'; operation = 'test'; property = 'actualState' }
        @{ version = '1.1.0'; operation = 'test'; property = 'actualState' }
        @{ version = '2.0.0'; operation = 'test'; property = 'actualState' }
    ) {
        param($version, $operation, $property)
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Test Version
              type: Test/Version
              apiVersion: $version
              properties:
                version: $version
"@
        $out = dsc -l trace config $operation -i $config_yaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $out.results[0].result.$property.version | Should -BeExactly $version
    }

    It "Version requirements '<req>' should return correct version" -TestCases @(
        @{ req = '>=1.0.0' ; expected = '2.0.0' }
        @{ req = '<=1.1.0' ; expected = '1.1.0' }
        @{ req = '<1.3' ; expected = '1.1.3' }
        @{ req = '>1,<=2.0.0' ; expected = '2.0.0' }
        @{ req = '>1.0.0,<2.0.0' ; expected = '1.1.3' }
        @{ req = '1'; expected = '1.1.3' }
        @{ req = '1.1' ; expected = '1.1.3' }
        @{ req = '^1.0' ; expected = '1.1.3' }
        @{ req = '~1.1' ; expected = '1.1.3' }
        @{ req = '*' ; expected = '2.0.0' }
        @{ req = '1.*' ; expected = '1.1.3' }
        @{ req = '2.1.0-preview.2' ; expected = '2.1.0-preview.2' }
    ) {
        param($req, $expected)
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Test Version
              type: Test/Version
              apiVersion: '$req'
              properties:
                version: $expected
"@
        $out = dsc -l trace config test -i $config_yaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $out.results[0].result.actualState.version | Should -BeExactly $expected
    }
}
