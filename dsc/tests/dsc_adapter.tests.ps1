# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for adapter support' {
    Context 'Adapter support single resource' {
        It 'Direct resource invocation for: <operation>' -TestCases @(
            @{ operation = 'get' },
            @{ operation = 'set' },
            @{ operation = 'test' },
            @{ operation = 'export' }
        ){
            param($operation)

            $out = dsc resource $operation -r Adapted/One -i '{"one":"1"}' 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log | Out-String)
            switch ($operation) {
                'get' {
                    $out.actualState.one | Should -BeExactly 'value1'
                }
                'set' {
                    $out.afterState.one | Should -BeExactly 'value1'
                }
                'test' {
                    $out.actualState.one | Should -BeExactly 'value1'
                    $out.inDesiredState | Should -BeFalse
                    $out.differingProperties | Should -Be @('one')
                }
                'export' {
                    $out.resources.count | Should -Be 2
                    $out.resources[0].type | Should -BeExactly 'Adapted/One'
                    $out.resources[0].name | Should -BeExactly 'first'
                    $out.resources[0].properties.one | Should -BeExactly 'first1'
                    $out.resources[1].type | Should -BeExactly 'Adapted/One'
                    $out.resources[1].name | Should -BeExactly 'second'
                    $out.resources[1].properties.one | Should -BeExactly 'second1'
                }
            }
        }

        It 'Config resource invocation for: <operation>' -TestCases @(
            @{ operation = 'get' },
            @{ operation = 'set' },
            @{ operation = 'test' },
            @{ operation = 'export' }
        ){
            param($operation)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Test
              type: Adapted/Two
              properties:
                two: '2'
"@
            $out = dsc config $operation -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log | Out-String)
            switch ($operation) {
                'get' {
                    $out.results.Count | Should -Be 1
                    $out.results[0].Name | Should -Be 'Test'
                    $out.results[0].type | Should -BeExactly 'Adapted/Two'
                    $out.results[0].result.actualState.two | Should -BeExactly 'value2' -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
                }
                'set' {
                    $out.results.Count | Should -Be 1
                    $out.results[0].Name | Should -Be 'Test'
                    $out.results[0].type | Should -BeExactly 'Adapted/Two'
                    $out.results[0].result.afterState.two | Should -BeExactly 'value2' -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
                }
                'test' {
                    $out.results.Count | Should -Be 1
                    $out.results[0].Name | Should -Be 'Test'
                    $out.results[0].type | Should -BeExactly 'Adapted/Two'
                    $out.results[0].result.actualState.two | Should -BeExactly 'value2' -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
                    $out.results[0].result.inDesiredState | Should -BeFalse
                    $out.results[0].result.differingProperties | Should -Be @('two') -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
                }
                'export' {
                    $out.resources.Count | Should -Be 2
                    $out.resources[0].Name | Should -Be 'first'
                    $out.resources[0].type | Should -BeExactly 'Adapted/Two'
                    $out.resources[0].properties.two | Should -BeExactly 'first2'
                    $out.resources[1].Name | Should -Be 'second'
                    $out.resources[1].type | Should -BeExactly 'Adapted/Two'
                    $out.resources[1].properties.two | Should -BeExactly 'second2'
                }
            }
        }


        It 'Specifying invalid adapter via metadata fails' {
            $config_yaml = @"
                `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
                resources:
                - name: Test
                  type: Microsoft.DSC.Debug/Echo
                  properties:
                    output: '1'
                  metadata:
                    Microsoft.DSC:
                      requireAdapter: InvalidAdapter/Invalid
"@
            $out = dsc config get -i $config_yaml 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.log | Out-String)
            $errorContent = Get-Content $TestDrive/error.log -Raw
            $errorContent | Should -Match "Adapter resource 'InvalidAdapter/Invalid' not found" -Because $errorContent
            $out | Should -BeNullOrEmpty -Because $errorContent
        }
    }
}
