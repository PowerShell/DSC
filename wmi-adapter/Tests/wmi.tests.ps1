# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WMI adapter resource tests' {

    BeforeAll {
        if ($IsWindows)
        {
            $OldPSModulePath  = $env:PSModulePath
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

    It 'List shows WMI resources' -Skip:(!$IsWindows){

        $r = dsc resource list *OperatingSystem* -a Microsoft.Windows/WMI
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.Count | Should -BeGreaterOrEqual 1
    }

    It 'Get works on an individual WMI resource' -Skip:(!$IsWindows){

        $r = dsc resource get -r root.cimv2/Win32_OperatingSystem
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.CreationClassName | Should -Be "Win32_OperatingSystem"
    }

    It 'Get works on a config with WMI resources' -Skip:(!$IsWindows){

        $r = Get-Content -Raw $configPath | dsc config get -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState[0].LastBootUpTime | Should -BeNullOrEmpty
        $res.results[0].result.actualState[0].Caption | Should -Not -BeNullOrEmpty
        $res.results[0].result.actualState[0].Version | Should -Not -BeNullOrEmpty
        $res.results[0].result.actualState[0].OSArchitecture | Should -Not -BeNullOrEmpty
    }

    It 'Example config works' -Skip:(!$IsWindows) {
        $configPath = Join-Path $PSScriptRoot '..\..\configurations\windows\windows_inventory.dsc.yaml'
        $r = dsc config get -f $configPath
        $LASTEXITCODE | Should -Be 0
        $r | Should -Not -BeNullOrEmpty
        $res = $r | ConvertFrom-Json
        $res.results[1].result.actualState[0].Name | Should -Not -BeNullOrEmpty
        $res.results[1].result.actualState[0].BootupState | Should -BeNullOrEmpty
        $res.results[1].result.actualState[1].Caption | Should -Not -BeNullOrEmpty
        $res.results[1].result.actualState[1].BuildNumber | Should -BeNullOrEmpty
        $res.results[1].result.actualState[4].AdapterType | Should -BeLike "Ethernet*"
    }
}
