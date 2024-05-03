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

    It 'Valid file path: <test>' -TestCases @(
        @{ test = 'absolute configuration'; config = (Join-Path $TestDrive 'include/osinfo_parameters.dsc.yaml'); parameters = $null }
        @{ test = 'absolute parameters'; config = 'include/osinfo_parameters.dsc.yaml'; parameters = (Join-Path $TestDrive 'include/osinfo.parameters.yaml') }
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
}
