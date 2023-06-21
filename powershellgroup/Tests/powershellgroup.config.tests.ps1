# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShellGroup resource tests' {

    BeforeAll {
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += ";" + $PSScriptRoot

        $configPath = Join-path $PSScriptRoot "testconfig2.yaml"
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    It 'Get works on config with class-based and script-based resources' -Skip:(!$IsWindows){
        
        $r = Get-Content -Raw $configPath | dsc config get
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actual_state[0].PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
        $res.results[0].result.actual_state[1].Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Test works on config with class-based and script-based resources' -Skip:(!$IsWindows){
        
        $r = Get-Content -Raw $configPath | dsc config test
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actual_state[0] | Should -Not -BeNull
        $res.results[0].result.actual_state[1] | Should -Not -BeNull
    }

    It 'Set works on config with class-based and script-based resources' -Skip:(!$IsWindows){
        
        $r = Get-Content -Raw $configPath | dsc config set
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results.result.after_state[0].RebootRequired | Should -Not -BeNull
        $res.results.result.after_state[1].RebootRequired | Should -Not -BeNull
    }
}
