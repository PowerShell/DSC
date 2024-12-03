# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter resource tests' {

    BeforeAll {
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot
        $pwshConfigPath = Join-path $PSScriptRoot "class_ps_resources.dsc.yaml"

        if ($IsLinux -or $IsMacOS) {
            $cacheFilePath = Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
        }
        else
        {
            $cacheFilePath = Join-Path $env:LocalAppData "dsc" "PSAdapterCache.json"
        }
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    BeforeEach {
        Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath
    }

    It 'Get works on config with class-based resources' {

        $r = Get-Content -Raw $pwshConfigPath | dsc config get -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState.result[0].properties.Prop1 | Should -BeExactly 'ValueForProp1'
        $res.results[0].result.actualState.result[0].properties.EnumProp | Should -BeExactly 'Expected'
    }

    It 'Test works on config with class-based resources' {

        $r = Get-Content -Raw $pwshConfigPath | dsc config test -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState.result[0] | Should -Not -BeNull
    }

    It 'Set works on config with class-based resources' {

        $r = Get-Content -Raw $pwshConfigPath | dsc config set -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results.result.afterState.result[0].type | Should -Be "TestClassResource/TestClassResource"
    }

    It 'Export works on config with class-based resources' {

        $yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
'@
        $out = $yaml | dsc config export -f -
        $LASTEXITCODE | Should -Be 0
        $res = $out | ConvertFrom-Json
        $res.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
        $res.'resources' | Should -Not -BeNullOrEmpty
        $res.resources[0].properties.result.count | Should -Be 5
        $res.resources[0].properties.result[0].Name | Should -Be "Object1"
        $res.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"
    }

    It 'Export fails when class-based resource does not implement' {
        $yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/NoExport
'@
        $out = $yaml | dsc config export -f - 2>&1 | Out-String
        $LASTEXITCODE | Should -Be 2
        $out | Should -Not -BeNullOrEmpty
        $out | Should -BeLike "*ERROR*Export method not implemented by resource 'TestClassResource/NoExport'*"
    }

    It 'Custom psmodulepath in config works' {

        $OldPSModulePath  = $env:PSModulePath
        Copy-Item -Recurse -Force -Path "$PSScriptRoot/TestClassResource" -Destination $TestDrive
        Rename-Item -Path "$PSScriptRoot/TestClassResource" -NewName "_TestClassResource"

        try {
            $psmp = "`$env:PSModulePath"+[System.IO.Path]::PathSeparator+$TestDrive
            $yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                psmodulepath: $psmp
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
"@
            $out = $yaml | dsc config export -f -
            $LASTEXITCODE | Should -Be 0
            $res = $out | ConvertFrom-Json
            $res.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
            $res.'resources' | Should -Not -BeNullOrEmpty
            $res.resources[0].properties.result.count | Should -Be 5
            $res.resources[0].properties.result[0].Name | Should -Be "Object1"
            $res.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"
        }
        finally {
            Rename-Item -Path "$PSScriptRoot/_TestClassResource" -NewName "TestClassResource"
            $env:PSModulePath = $OldPSModulePath
        }
    }

    It 'DSCConfigRoot macro is working when config is from a file' {

        $yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
                  properties:
                    Name: "[envvar('DSC_CONFIG_ROOT')]"
"@

        $config_path = "$TestDrive/test_config.dsc.yaml"
        $yaml | Set-Content -Path $config_path

        $out = dsc config get --file $config_path
        $LASTEXITCODE | Should -Be 0
        $res = $out | ConvertFrom-Json
        $res.results.result.actualState.result.properties.Name | Should -Be $TestDrive
        $res.results.result.actualState.result.properties.Prop1 | Should -Be $TestDrive
    }

    It 'DSC_CONFIG_ROOT env var is cwd when config is piped from stdin' {

        $yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
                  properties:
                    Name: "[envvar('DSC_CONFIG_ROOT')]"
"@
        $out = $yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.result[0].properties.Name | Should -BeExactly (Get-Location).Path
    }
}
