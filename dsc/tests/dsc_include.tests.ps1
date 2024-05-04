# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Include tests' {
    BeforeAll {
        $includePath = New-Item -ItemType Directory -Path (Join-Path $TestDrive 'include')
        Copy-Item (Join-Path $PSScriptRoot '../examples/osinfo_parameters.dsc.yaml') -Destination $includePath
        $osinfoConfigPath = Get-Item (Join-Path $includePath 'osinfo_parameters.dsc.yaml')
        Copy-Item (Join-Path $PSScriptRoot '../examples/osinfo.parameters.yaml') -Destination $includePath
        $osinfoParametersConfigPath = Get-Item (Join-Path $includePath 'osinfo.parameters.yaml')

        $logPath = Join-Path $TestDrive 'stderr.log'

        $includeConfig = @'
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                output: Hello World
'@
    }

    It 'Include config with default parameters' {
        $config = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: osinfo
              type: Microsoft.DSC/Include
              properties:
                configurationFile: include/osinfo_parameters.dsc.yaml
"@
        $configPath = Join-Path $TestDrive 'config.dsc.yaml'
        $config | Set-Content -Path $configPath
        $out = dsc config get -p $configPath | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($IsWindows) {
            $expectedOS = 'Windows'
        } elseif ($IsLinux) {
            $expectedOS = 'Linux'
        } else {
            $expectedOS = 'macOS'
        }
        $out.results[0].result[0].result.actualState.family | Should -Be $expectedOS
    }

    It 'Include config with parameters file' {
        $config = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: osinfo
              type: Microsoft.DSC/Include
              properties:
                configurationFile: include/osinfo_parameters.dsc.yaml
                parametersFile: include/osinfo.parameters.yaml
"@
        $configPath = Join-Path $TestDrive 'config.dsc.yaml'
        $config | Set-Content -Path $configPath
        $out = dsc config get -p $configPath | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($IsWindows) {
            $expectedOS = 'Windows'
        } elseif ($IsLinux) {
            $expectedOS = 'Linux'
        } else {
            $expectedOS = 'macOS'
        }
        $out.results[0].result[0].result.actualState.family | Should -Be $expectedOS
    }

    It 'Invalid file path: <test>' -TestCases @(
        @{ test = 'non-existing configuration'; config = 'include/non-existing.dsc.yaml'; parameters = $null }
        @{ test = 'non-existing parameters'; config = 'include/osinfo_parameters.dsc.yaml'; parameters = 'include/non-existing.parameters.yaml' }
        @{ test = 'configuration referencing parent directory'; config = '../include/osinfo_parameters.dsc.yaml'; parameters = $null }
        @{ test = 'parameters referencing parent directory'; config = 'include/osinfo_parameters.dsc.yaml'; parameters = '../include/non-existing.parameters.yaml' }
    ) {
        param($config, $parameters)

        $configYaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: osinfo
              type: Microsoft.DSC/Include
              properties:
                configurationFile: $config
                parametersFile: $parameters
"@

        $configPath = Join-Path $TestDrive 'config.dsc.yaml'
        $configYaml | Set-Content -Path $configPath
        $out = dsc config get -p $configPath 2> $logPath
        $LASTEXITCODE | Should -Be 2
        $log = Get-Content -Path $logPath -Raw
        $log | Should -BeLike "*ERROR*"
    }

    It 'Valid absolute file path: <test>' -TestCases @(
        @{ test = 'configuration'; config = 'include/osinfo_parameters.dsc.yaml'; parameters = $null }
        @{ test = 'parameters'; config = 'include/osinfo_parameters.dsc.yaml'; parameters = 'include/osinfo.parameters.yaml' }
    ) {
        param($test, $config, $parameters)

        if ($test -eq 'configuration') {
            $config = Join-Path $TestDrive $config
        } elseif ($test -eq 'parameters') {
            $parameters = Join-Path $TestDrive $parameters
        } else {
            throw "Invalid test case: $test"
        }

        $configYaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: osinfo
              type: Microsoft.DSC/Include
              properties:
                configurationFile: $config
                parametersFile: $parameters
"@

        $configPath = Join-Path $TestDrive 'config.dsc.yaml'
        $configYaml | Set-Content -Path $configPath
        $out = dsc config get -p $configPath | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($IsWindows) {
            $expectedOS = 'Windows'
        } elseif ($IsLinux) {
            $expectedOS = 'Linux'
        } else {
            $expectedOS = 'macOS'
        }
        $out.results[0].result[0].result.actualState.family | Should -Be $expectedOS
    }

    It 'Multiple includes' {
        $echoConfig = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: one
              type: Test/Echo
              properties:
                output: one
'@

        $echoConfigPath = Join-Path $TestDrive 'echo.dsc.yaml'
        $echoConfig | Set-Content -Path $echoConfigPath

        $nestedIncludeConfig = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: nested
              type: Microsoft.DSC/Include
              properties:
                configurationFile: $echoConfigPath
"@

        $nestedIncludeConfigPath = Join-Path $TestDrive 'nested_include.dsc.yaml'
        $nestedIncludeConfig | Set-Content -Path $nestedIncludeConfigPath

        $includeConfig = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: include
              type: Microsoft.DSC/Include
              properties:
                configurationFile: $echoConfigPath
            - name: include nested
              type: Microsoft.DSC/Include
              properties:
                configurationFile: $nestedIncludeConfigPath
"@

        $out = $includeConfig | dsc -l trace config get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result[0].result.actualState.output | Should -Be 'one'
        $out.results[1].result[0].name | Should -Be 'nested'
        $out.results[1].result[0].type | Should -Be 'Microsoft.DSC/Include'
        $out.results[1].result[0].result[0].name | Should -Be 'one'
        $out.results[1].result[0].result[0].type | Should -Be 'Test/Echo'
        $out.results[1].result[0].result[0].result[0].actualState.output | Should -Be 'one'
    }
}
