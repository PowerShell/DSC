# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter resource tests' {

    BeforeAll {
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += ";" + $PSScriptRoot

        $configPath = Join-path $PSScriptRoot "class_ps_resources.dsc.yaml"
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    It 'Get works on config with class-based and script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $configPath | dsc config get
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState[0].PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
        $res.results[0].result.actualState[1].Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Test works on config with class-based and script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $configPath | dsc config test
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState[0] | Should -Not -BeNull
        $res.results[0].result.actualState[1] | Should -Not -BeNull
    }

    It 'Set works on config with class-based and script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $configPath | dsc config set
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results.result.afterState[0].RebootRequired | Should -Not -BeNull
        $res.results.result.afterState[1].RebootRequired | Should -Not -BeNull
    }

    It 'Export works on config with class-based resources' -Skip:(!$IsWindows){

        $yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: PSTestModule/TestClassResource
'@
        $out = $yaml | dsc config export
        $LASTEXITCODE | Should -Be 0
        $res = $out | ConvertFrom-Json
        $res.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json'
        $res.'resources' | Should -Not -BeNullOrEmpty
        $res.resources.count | Should -Be 5
        $res.resources[0].properties.Name | Should -Be "Object1"
        $res.resources[0].properties.Prop1 | Should -Be "Property of object1"
    }

    It 'Custom psmodulepath in config works' -Skip:(!$IsWindows){

        $OldPSModulePath  = $env:PSModulePath
        Copy-Item -Recurse -Force -Path "$PSScriptRoot/PSTestModule" -Destination $TestDrive
        Rename-Item -Path "$PSScriptRoot/PSTestModule" -NewName "_PSTestModule"

        try {
            $yaml = @"
                `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
                resources:
                - name: Working with class-based resources
                  type: Microsoft.DSC/PowerShell
                  properties:
                    psmodulepath: `$env:PSModulePath;$TestDrive
                    resources:
                    - name: Class-resource Info
                      type: PSTestModule/TestClassResource
"@
            $out = $yaml | dsc config export
            $LASTEXITCODE | Should -Be 0
            $res = $out | ConvertFrom-Json
            $res.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json'
            $res.'resources' | Should -Not -BeNullOrEmpty
            $res.resources.count | Should -Be 5
            $res.resources[0].properties.Name | Should -Be "Object1"
            $res.resources[0].properties.Prop1 | Should -Be "Property of object1"
        }
        finally {
            Rename-Item -Path "$PSScriptRoot/_PSTestModule" -NewName "PSTestModule"
            $env:PSModulePath = $OldPSModulePath
        }
    }

    It 'DSCConfigRoot macro is working when config is from a file' -Skip:(!$IsWindows){

        $yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: PSTestModule/TestClassResource
                  properties:
                    Name: "[envvar('DSCConfigRoot')]"
"@

        $config_path = "$TestDrive/test_config.dsc.yaml"
        $yaml | Set-Content -Path $config_path

        $out = dsc config get --path $config_path
        $LASTEXITCODE | Should -Be 0
        $res = $out | ConvertFrom-Json
        $res.results[0].result.actualState.Name | Should -Be $TestDrive
        $res.results[0].result.actualState.Prop1 | Should -Be $TestDrive
    }

    It 'DSCConfigRoot macro is empty when config is piped from stdin' -Skip:(!$IsWindows){

        $yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: PSTestModule/TestClassResource
                  properties:
                    Name: "[envvar('DSCConfigRoot')]"
"@
        $out = $yaml | dsc config get
        $LASTEXITCODE | Should -Be 0
        $res = $out | ConvertFrom-Json
        $res.results[0].result.actualState.Name | Should -Be ""
        $res.results[0].result.actualState.Prop1 | Should -Be ""
    }
}
