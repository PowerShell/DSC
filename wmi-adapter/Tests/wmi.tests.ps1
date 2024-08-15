# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WMI adapter resource tests' {

    BeforeAll {
        if ($IsWindows)
        {
            $OldPSModulePath = $env:PSModulePath
            $env:PSModulePath += ";" + $PSScriptRoot

            $configPath = Join-path $PSScriptRoot "test_wmi_config.dsc.yaml"
        }
    }
    AfterAll {
        if ($IsWindows)
        {
            $env:PSModulePath = $OldPSModulePath
        }
    }

    Context 'List WMI resources' {
        It 'List shows WMI resources' -Skip:(!$IsWindows) {

            $r = dsc resource list *OperatingSystem* -a Microsoft.Windows/WMI
            $LASTEXITCODE | Should -Be 0
            $res = $r | ConvertFrom-Json
            $res.Count | Should -BeGreaterOrEqual 1
        }   
    }

    Context 'Get WMI resources' {
        It 'Get works on an individual WMI resource' -Skip:(!$IsWindows) {

            $r = dsc resource get -r root.cimv2/Win32_OperatingSystem
            $LASTEXITCODE | Should -Be 0
            $res = $r | ConvertFrom-Json
            $res.actualState.result.type | Should -BeLike "*Win32_OperatingSystem"
        }
    
        It 'Get works on a config with WMI resources' -Skip:(!$IsWindows) {
    
            $r = Get-Content -Raw $configPath | dsc config get
            $LASTEXITCODE | Should -Be 0
            $res = $r | ConvertFrom-Json
            $res.results.result.actualstate.result[0].properties.value.LastBootUpTime | Should -Not -BeNull
            $res.results.result.actualstate.result[0].properties.value.Caption | Should -Not -BeNull
            $res.results.result.actualstate.result[0].properties.value.NumberOfProcesses | Should -Not -BeNull
        }
    
        It 'Example config works' -Skip:(!$IsWindows) {
            $configPath = Join-Path $PSScriptRoot '..\..\dsc\examples\wmi.dsc.yaml'
            $r = dsc config get -p $configPath
            $LASTEXITCODE | Should -Be 0
            $r | Should -Not -BeNullOrEmpty
            $res = $r | ConvertFrom-Json
            $res.results.result.actualstate.result[0].properties.value.Model | Should -Not -BeNullOrEmpty
            $res.results.result.actualstate.result[0].properties.value.Description | Should -Not -BeNullOrEmpty
        }
    }

    # TODO: work on set test
    # Context "Set WMI resources" {
    # }

    # TODO: get export working
    # Context "Export WMI resources" {
    #     It 'Exports all resources' -Skip:(!$IsWindows) {
    #         $r = dsc resource export -r root.cimv2/Win32_Process
    #         $LASTEXITCODE | Should -Be 0
    #         $res = $r | ConvertFrom-Json
    #         $res.resources[0].properties.result.count | Should -BeGreaterThan 1
    #         $json.resources[0].properties.result.properties.value | Should -not -BeNullOrEmpty
    #     }
    # }
}
