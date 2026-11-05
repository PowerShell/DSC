# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'DSC Actions' {
    It 'Actions should be listed' {
        $out = dsc action list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.count | Should -Be 2
        $out[0].type | Should -BeExactly 'Test/Action2'
        $out[0].version | Should -BeExactly '0.1.0'
        $out[0].inputSchema | Should -BeNullOrEmpty
        $out[0].outputSchema | Should -BeNullOrEmpty
        $out[0].author | Should -BeExactly 'DSC Team'
        $out[0].description | Should -Not -BeNullOrEmpty
        $out[0].supportedOperations | Should -Be @('set')
        $out[0].manifest | Should -Not -BeNullOrEmpty
        $out[1].type | Should -BeExactly 'Test/Action'
        $out[1].version | Should -BeExactly '0.1.0'
        $out[1].inputSchema | Should -Not -BeNullOrEmpty
        $out[1].outputSchema | Should -Not -BeNullOrEmpty
        $out[1].author | Should -BeExactly 'DSC Team'
        $out[1].description | Should -Not -BeNullOrEmpty
        $out[1].supportedOperations | Should -Be @('get', 'set', 'test')
        $out[1].manifest | Should -Not -BeNullOrEmpty
    }

    It 'Action should be invokable with <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'Action Test'
                    type = 'Test/Action'
                    properties = @{
                        inputText = 'Hello world'
                    }
                }
                @{
                    name = 'Echo'
                    type = 'Microsoft.DSC.Debug/Echo'
                    properties = @{
                        output = 'Echo output'
                    }
                }
            )
        }

        $out = dsc config $operation -i ($config | ConvertTo-Json -Depth 5) | ConvertFrom-Json -Depth 5
        $LASTEXITCODE | Should -Be 0
        $property = if ($operation -eq 'set') {
            'afterState'
        } else {
            'actualState'
        }
        $out.results[0].type | Should -BeExactly 'Test/Action'
        $out.results[0].result.$property.outputText | Should -BeExactly 'Hello world' -Because ($out | ConvertTo-Json -Depth 5 | Out-String)
        $out.results[1].type | Should -BeExactly 'Microsoft.DSC.Debug/Echo'
        $out.results[1].result.$property.output | Should -BeExactly 'Echo output'
    }

    It 'Action with version works correct: <expected>' -TestCases @(
        @{ version = '0.1.0'; expected = 'found' }
        @{ version = '0.2.0'; expected = 'error' }
    ) {
        param ($version, $expected)

        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'Action Test'
                    type = 'Test/Action'
                    requireVersion = "=$version"
                    properties = @{
                        inputText = 'Hello world'
                    }
                }
            )
        }

        $out = dsc config get -i ($config | ConvertTo-Json -Depth 5) 2> $TestDrive/error.log | ConvertFrom-Json -Depth 5
        $errorlog = Get-Content $TestDrive/error.log -Raw
        if ($expected -eq 'found') {
            $LASTEXITCODE | Should -Be 0 -Because $errorLog
            $out.results[0].type | Should -BeExactly 'Test/Action'
        } else {
            $LASTEXITCODE | Should -Be 2
            $errorlog | Should -BeLike '*ERROR*Resource not found: Test/Action =0.2.0*'
        }
    }

    It 'Action JSON schema is validated' {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'Action Test'
                    type = 'Test/Action'
                    properties = @{
                        invalidProperty = 'Hello world'
                    }
                }
            )
        }

        dsc config get -i ($config | ConvertTo-Json -Depth 5) 2> $TestDrive/error.log
        $errorlog = Get-Content $TestDrive/error.log -Raw
        $LASTEXITCODE | Should -Be 2 -Because $errorlog
        $errorlog | Should -BeLike '*ERROR Schema: "inputText" is a required property*'
    }
}
