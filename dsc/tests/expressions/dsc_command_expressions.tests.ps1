# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Expressions as command resource arguments' {
    It 'Expressions can be passed in resource manifest as arguments for <operation>' -TestCases @(
        @{ operation = 'get'; property = 'actualState' }
        @{ operation = 'set'; property = 'afterState' }
        @{ operation = 'test'; property = 'actualState' }
        @{ operation = 'export'; property = 'actualState' }
    ){
        param($operation, $property)

        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'Test'
                    type = 'Test/ExpressionArg'
                    properties = @{
                        first = "Hello"
                        second = "World"
                        third = "Bye"
                    }
                }
            )
        }

        $out = dsc -l trace config $operation -i ($config | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
        $errorLog = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorLog
        if ($operation -eq 'export') {
            $out.resources[0].properties.first | Should -BeExactly 'Hello' -Because ($out | ConvertTo-Json -Depth 10)
            $out.resources[0].properties.second | Should -BeExactly 'World'
            $out.resources[0].properties.third | Should -BeExactly 'Bye'
        } else {
            $out.results[0].result.$property.first | Should -BeExactly 'Hello' -Because ($out | ConvertTo-Json -Depth 10)
            $out.results[0].result.$property.second | Should -BeExactly 'World'
            $out.results[0].result.$property.third | Should -BeExactly 'Bye'
        }
    }

    It 'Mandatory arg without value should be empty string' {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'Test'
                    type = 'Test/ExpressionArg'
                    properties = @{
                        first = ''
                        second = ''
                        third = ''
                    }
                }
            )
        }

        $out = dsc -l trace config get -i ($config | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
        $errorLog = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorLog
        $out.results[0].result.actualState.first | Should -BeExactly ''
        $out.results[0].result.actualState.second | Should -BeExactly ''
        $out.results[0].result.actualState.third | Should -BeExactly ''
    }

    It 'Optional arg without supporting property should be ignored' {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'Test'
                    type = 'Test/ExpressionArg'
                    properties = @{
                        first = '1'
                        second = '2'
                    }
                }
            )
        }

        $out = dsc -l trace config get -i ($config | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
        $errorLog = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorLog
        $out.results[0].result.actualState.first | Should -BeExactly '1'
        $out.results[0].result.actualState.second | Should -BeExactly '2'
    }
}
