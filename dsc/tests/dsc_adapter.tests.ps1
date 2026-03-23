# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for adapter support' {
    Context 'Adapter support single resource' {
        BeforeAll {
            $OldPSModulePath = $env:PSModulePath
            $env:PSModulePath += [System.IO.Path]::PathSeparator + (Resolve-Path "$PSScriptRoot/../../adapters/powershell/Tests")
        }

        AfterAll {
            $env:PSModulePath = $OldPSModulePath
        }


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
                  type: Test/Invalid
                  properties:
                    output: '1'
                  directives:
                    requireAdapter: InvalidAdapter/Invalid
"@
            $out = dsc config get -i $config_yaml 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.log | Out-String)
            $errorContent = Get-Content $TestDrive/error.log -Raw
            $errorContent | Should -Match "Adapter not found: InvalidAdapter/Invalid" -Because $errorContent
            $out | Should -BeNullOrEmpty -Because $errorContent
        }

        It 'Specifying two adapters for same resource works' {
            $config_yaml = @'
                $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
                resources:
                - name: Test
                  type: TestClassResource/TestClassResource
                  properties:
                    Name: 'Hello'
                  directives:
                    requireAdapter: Microsoft.DSC/PowerShell
                - name: Test2
                  type: TestClassResource/TestClassResource
                  properties:
                    Name: 'Bye'
                  directives:
                    requireAdapter: Microsoft.Adapter/PowerShell
'@
            $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log | Out-String)
            $out.results.Count | Should -Be 2
            $out.results[0].type | Should -BeExactly 'TestClassResource/TestClassResource'
            $out.results[0].Name | Should -Be 'Test'
            $out.results[0].result.actualState.Name | Should -BeExactly 'Hello'
            $out.results[1].type | Should -BeExactly 'TestClassResource/TestClassResource'
            $out.results[1].Name | Should -Be 'Test2'
            $out.results[1].result.actualState.Name | Should -BeExactly 'Bye'
            "$TestDrive/error.log" | Should -FileContentMatch "Invoking get for 'Microsoft.DSC/PowerShell'" -Because (Get-Content $TestDrive/error.log | Out-String)
            "$TestDrive/error.log" | Should -FileContentMatch "Invoking get for 'Microsoft.Adapter/PowerShell'" -Because (Get-Content $TestDrive/error.log | Out-String)
        }
    }

    Context 'Adapted resource manifests' {
        It 'Adapted resource are found in individual manifest' {
            $out = dsc resource list 'Adapted/Three' 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log | Out-String)
            $out.count | Should -Be 1
            $out.type | Should -BeExactly 'Adapted/Three'
            $out.kind | Should -BeExactly 'resource'
            $out.capabilities | Should -Be @('get', 'set', 'test', 'export')
            $parent = (Split-Path -Path (Get-Command dsc).Source -Parent)
            $expectedPath = Join-Path -Path $parent -ChildPath 'adaptedTest.dsc.adaptedResource.json'
            $out.path | Should -BeExactly $expectedPath
            $out.directory | Should -BeExactly $parent
            $out.requireAdapter | Should -BeExactly 'Test/Adapter'
            $out.schema.embedded | Should -Not -BeNullOrEmpty
        }

        It 'Adapted resource found in manifest list' {
            $out = dsc resource list 'Adapted/Two' 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log | Out-String)
            $out.count | Should -Be 1
            $out.type | Should -BeExactly 'Adapted/Two'
            $out.kind | Should -BeExactly 'resource'
            $out.capabilities | Should -Be @('get', 'set', 'test', 'export')
            $parent = (Split-Path -Path (Get-Command dsc).Source -Parent)
            $expectedPath = Join-Path -Path $parent -ChildPath 'adaptedTest.dsc.adaptedResource.json'
            $out.path | Should -BeExactly $expectedPath
            $out.directory | Should -BeExactly $parent
            $out.requireAdapter | Should -BeExactly 'Test/Adapter'
            $out.schema | Should -Not -BeNullOrEmpty
        }

        It 'Adapted resource with condition false should not be returned' {
            $out = dsc -l debug resource list 'Adapted/Four' 2>$TestDrive/error.log
            $errorLog = Get-Content $TestDrive/error.log -Raw
            $LASTEXITCODE | Should -Be 0 -Because $errorLog
            $out | Should -BeNullOrEmpty -Because $errorLog
            $errorLog | Should -Match "Condition '.*?' not met, skipping manifest at .*? for resource 'Adapted/Four" -Because $errorLog
        }

        It 'Invoking <operation> on adapted resource works' -TestCases @(
            @{ operation = 'get' },
            @{ operation = 'set' },
            @{ operation = 'test' },
            @{ operation = 'export' }
        ){
            param($operation)
            $out = dsc resource $operation -r Adapted/Three -i '{"one":"3"}' 2>$TestDrive/error.log | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log | Out-String)
            $parent = (Split-Path -Path (Get-Command dsc).Source -Parent)
            $expectedPath = Join-Path -Path $parent -ChildPath 'adaptedTest.dsc.adaptedResource.json'
            switch ($operation) {
                'get' {
                    $out.actualState.one | Should -BeExactly 'value3'
                    $out.actualState.path | Should -BeExactly $expectedPath
                }
                'set' {
                    $out.afterState.one | Should -BeExactly 'value3'
                    $out.afterState.path | Should -BeExactly $expectedPath
                }
                'test' {
                    $out.actualState.one | Should -BeExactly 'value3'
                    $out.actualState.path | Should -BeExactly $expectedPath
                    $out.inDesiredState | Should -BeFalse
                    $out.differingProperties | Should -Be @('one')
                }
                'export' {
                    $out.resources.count | Should -Be 2
                    $out.resources[0].type | Should -BeExactly 'Adapted/Three'
                    $out.resources[0].name | Should -BeExactly 'first'
                    $out.resources[0].properties.one | Should -BeExactly 'first3'
                    $out.resources[1].type | Should -BeExactly 'Adapted/Three'
                    $out.resources[1].name | Should -BeExactly 'second'
                    $out.resources[1].properties.one | Should -BeExactly 'second3'
                }
            }
        }

        It 'Deprecated adapted resource shows message' {
            try {
                $dscHome = Split-Path (Get-Command dsc).Source -Parent
                $env:DSC_RESOURCE_PATH = (Join-Path -Path $dscHome -ChildPath 'deprecated') + [System.IO.Path]::PathSeparator + $dscHome
                $out = dsc resource get -r Adapted/Deprecated -i '{}' 2>$TestDrive/error.log | ConvertFrom-Json
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log | Out-String)
                $out | Should -Not -BeNullOrEmpty
                (Get-Content $TestDrive/error.log -Raw) | Should -Match "Resource 'Adapted/Deprecated' is deprecated: This adapted resource is deprecated" -Because (Get-Content $TestDrive/error.log | Out-String)
            } finally {
                $env:DSC_RESOURCE_PATH = $null
            }
        }
    }
}
