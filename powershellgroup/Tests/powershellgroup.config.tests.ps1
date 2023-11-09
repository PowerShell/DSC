# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShellGroup resource tests' {

    BeforeAll {
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += ";" + $PSScriptRoot

        $configPath = Join-path $PSScriptRoot "class_ps_resources.dsc.yaml"
        $multiScriptResourceConfigPath = Join-path $PSScriptRoot "script_ps_resource.dsc.yaml"
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    It 'Get works on config with class-based and script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $configPath | dsc config get
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState.resources[0].properties.PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
        $res.results[0].result.actualState.resources[1].properties.Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Test works on config with class-based and script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $configPath | dsc config test
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState.resources[0] | Should -Not -BeNull
        $res.results[0].result.actualState.resources[1] | Should -Not -BeNull
    }

    It 'Set works on config with class-based and script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $configPath | dsc config set
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results.result.afterState.resources[0].properties.RebootRequired | Should -Not -BeNull
        $res.results.result.afterState.resources[1].properties.RebootRequired | Should -Not -BeNull
    }

    It 'Get works on config with multiple script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $multiScriptResourceConfigPath | dsc config get
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState.resources[0].properties.Name | Should -BeExactly 'bits'
        $res.results[0].result.actualState.resources[0].properties.State | Should -BeExactly 'running'
        $res.results[0].result.actualState.resources[1].properties.Name | Should -BeExactly 'spooler'
        $res.results[0].result.actualState.resources[1].properties.State | Should -BeExactly 'running'
    }

    It 'Test works on config with multiple script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $multiScriptResourceConfigPath | dsc config test
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState.resources[0] | Should -Not -BeNull
        $res.results[0].result.actualState.resources[1] | Should -Not -BeNull
    }

    It 'Set works on config with multiple script-based resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $multiScriptResourceConfigPath | dsc config set
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results.result.afterState.resources[0].properties.RebootRequired | Should -Not -BeNull
        $res.results.result.afterState.resources[1].properties.RebootRequired | Should -Not -BeNull
    }
}
