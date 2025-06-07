# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc config get tests' {
    It 'can successfully get config with multiple registry resource instances: <config>' -Skip:(!$IsWindows) -TestCases @(
        @{ config = 'osinfo_registry.dsc.json' }
        @{ config = 'osinfo_registry.dsc.yaml' }
    ) {
        param($config)
        $jsonPath = Join-Path $PSScriptRoot '../examples' $config
        $config = Get-Content $jsonPath -Raw
        $out = $config | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.hadErrors | Should -BeFalse
        $out.results.Count | Should -Be 3
        $out.results[0].Name | Should -Be 'os'
        $out.results[0].type | Should -BeExactly 'Microsoft/OSInfo'
        $out.results[0].result.actualState.family | Should -BeExactly 'Windows'
        $out.results[1].Name | Should -Be 'windows product name'
        $out.results[1].type | Should -BeExactly 'Microsoft.Windows/Registry'
        $out.results[1].result.actualState.valueData.String | Should -BeLike 'Windows*'
        $out.results[2].Name | Should -Be 'system root'
        $out.results[2].type | Should -BeExactly 'Microsoft.Windows/Registry'
        $out.results[2].result.actualState.valueData.String | Should -BeExactly $env:SystemRoot
    }

    It 'will fail if resource schema does not match' -Skip:(!$IsWindows) {
        $jsonPath = Join-Path $PSScriptRoot '../examples/invalid_schema.dsc.yaml'
        $testError = & {dsc config get -f $jsonPath 2>&1}
        $testError[0] | Should -match 'Schema:'
        $LASTEXITCODE | Should -Be 2
    }

    It 'can accept the use of --output-format as a subcommand' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: hello
"@
        $result = $config_yaml | dsc config get --output-format pretty-json -f - | ConvertFrom-Json
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $result.results[0].Name | Should -Be 'Echo'
        $result.results[0].type | Should -BeExactly 'Microsoft.DSC.Debug/Echo'
        $result.results[0].result.actualState.output | Should -Be 'hello'
        $result.metadata.'Microsoft.DSC'.version | Should -BeLike '3.*'
        $result.metadata.'Microsoft.DSC'.operation | Should -BeExactly 'get'
        $result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'actual'
        $result.metadata.'Microsoft.DSC'.startDatetime | Should -Not -BeNullOrEmpty
        $result.metadata.'Microsoft.DSC'.endDatetime | Should -Not -BeNullOrEmpty
        $result.metadata.'Microsoft.DSC'.duration | Should -Not -BeNullOrEmpty
        $result.metadata.'Microsoft.DSC'.securityContext | Should -Not -BeNullOrEmpty
        $LASTEXITCODE | Should -Be 0
    }

    It 'json progress for config subcommand' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo 1
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: hello
            - name: Echo 2
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: world
"@
        $config_yaml | dsc --progress-format json config get -f - 2> $TestDrive/ErrorStream.txt
        $LASTEXITCODE | Should -Be 0
        $lines = Get-Content $TestDrive/ErrorStream.txt
        $ProgressMessagesFound = $false
        $InstanceOneFound = $false
        $InstanceTwoFound = $false
        foreach ($line in $lines) {
            $jp = $line | ConvertFrom-Json
            if ($jp.activity) { # if line is a progress message
                $jp.id | Should -Not -BeNullOrEmpty
                $jp.totalItems | Should -Not -BeNullOrEmpty
                $jp.completedItems | Should -Not -BeNullOrEmpty
                $ProgressMessagesFound = $true
            }

            if ($null -ne $jp.result -and $jp.resourceType -eq 'Microsoft.DSC.Debug/Echo') {
                if ($jp.resourceName -eq 'Echo 1') {
                    $InstanceOneFound = $true
                    $jp.result.actualState.output | Should -BeExactly 'hello'
                } elseif ($jp.resourceName -eq 'Echo 2') {
                    $InstanceTwoFound = $true
                    $jp.result.actualState.output | Should -BeExactly 'world'
                }
            }
        }
        $ProgressMessagesFound | Should -BeTrue
        $InstanceOneFound | Should -BeTrue
        $InstanceTwoFound | Should -BeTrue
    }

    It 'json progress returns correctly for failed resource' {
        $config_yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Echo 1
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: hello
            - name: ErrorTest
              type: Test/ExitCode
              properties:
                exitCode: 8
'@
        dsc --progress-format json --trace-format json config get -i $config_yaml 2> $TestDrive/ErrorStream.txt
        $LASTEXITCODE | Should -Be 2
        $lines = Get-Content $TestDrive/ErrorStream.txt
        $ProgressMessagesFound = $false
        $InstanceOneFound = $false
        $InstanceTwoFound = $false
        foreach ($line in $lines) {
            $jp = $line | ConvertFrom-Json
            if ($jp.activity) { # if line is a progress message
                $jp.id | Should -Not -BeNullOrEmpty
                $jp.totalItems | Should -Not -BeNullOrEmpty
                $jp.completedItems | Should -Not -BeNullOrEmpty
                $ProgressMessagesFound = $true
            }

            if ($null -ne $jp.result -and $jp.resourceType -eq 'Microsoft.DSC.Debug/Echo') {
                if ($jp.resourceName -eq 'Echo 1') {
                    $InstanceOneFound = $true
                    $jp.result.actualState.output | Should -BeExactly 'hello'
                    $jp.failed | Should -BeNullOrEmpty
                }
            }
            elseif ($null -ne $jp.failure -and $jp.resourceType -eq 'Test/ExitCode') {
                if ($jp.resourceName -eq 'ErrorTest') {
                    $InstanceTwoFound = $true
                    $jp.result | Should -BeNullOrEmpty
                    $jp.failure.exitCode | Should -Be 8
                    $jp.failure.message | Should -Not -BeNullOrEmpty
                }
            }
        }
        $ProgressMessagesFound | Should -BeTrue
        $InstanceOneFound | Should -BeTrue
        $InstanceTwoFound | Should -BeTrue
    }

    It 'contentVersion is ignored' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            contentVersion: 1.0.0.0
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: hello
"@
        $result = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $result.results[0].Name | Should -Be 'Echo'
        $result.results[0].type | Should -BeExactly 'Microsoft.DSC.Debug/Echo'
        $result.results[0].result.actualState.output | Should -Be 'hello'
        $LASTEXITCODE | Should -Be 0
    }

    It 'no properties is supported' {
        $config_yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: OS
              type: Microsoft/OSInfo
'@
        $result = dsc config get -i $config_yaml | ConvertFrom-Json
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $result.results[0].Name | Should -Be 'OS'
        $result.results[0].type | Should -BeExactly 'Microsoft/OSInfo'
        $result.results[0].result.actualState.family | Should -BeIn @('Windows', 'Linux', 'macOS')
        $LASTEXITCODE | Should -Be 0
    }
}
