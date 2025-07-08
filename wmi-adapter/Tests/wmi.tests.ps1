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
        $res.results[0].result.actualState.result[0].properties.LastBootUpTime | Should -BeNullOrEmpty
        $res.results[0].result.actualState.result[0].properties.Caption | Should -Not -BeNullOrEmpty
        $res.results[0].result.actualState.result[0].properties.Version | Should -Not -BeNullOrEmpty
        $res.results[0].result.actualState.result[0].properties.OSArchitecture | Should -Not -BeNullOrEmpty
    }

    It 'Example config works' -Skip:(!$IsWindows) {
        $configPath = Join-Path $PSScriptRoot '..\..\configurations\windows\windows_inventory.dsc.yaml'
        $r = dsc config get -f $configPath
        $LASTEXITCODE | Should -Be 0
        $r | Should -Not -BeNullOrEmpty
        $res = $r | ConvertFrom-Json
        $res.results[1].result.actualState.result[0].properties.Name | Should -Not -BeNullOrEmpty
        $res.results[1].result.actualState.result[0].properties.BootupState | Should -BeNullOrEmpty
        $res.results[1].result.actualState.result[1].properties.Caption | Should -Not -BeNullOrEmpty
        $res.results[1].result.actualState.result[1].properties.BuildNumber | Should -BeNullOrEmpty
        $res.results[1].result.actualState.result[4].properties.AdapterType | Should -BeLike "Ethernet*"
    }

    It 'Throws error when methodName is missing on set' -Skip:(!$IsWindows) {
        '{"Name":"wuauserv"}' | dsc -l trace resource set -r 'root.cimv2/Win32_Service' -f - 2> $TestDrive\tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatch "'methodName' property is required for invoking a WMI/CIM method."
    }

    It 'Throws error when methodName is set but parameters are missing on set' -Skip:(!$IsWindows) {
        '{"Name":"wuauserv", "methodName":"StartService"}' | dsc -l trace resource set -r 'root.cimv2/Win32_Service' -f - 2> $TestDrive\tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatch "'parameters' property is required for invoking a WMI/CIM method."
    }

    It 'Set works on a WMI resource with methodName and parameters' -Skip:(!$IsWindows) {
        BeforeAll {
            $script:service = Get-Service -Name wuauserv -ErrorAction Ignore 

            if ($service -and $service.Status -eq 'Running') {
                $service.Stop()
            }
        }
        

        $r = '{"Name":"wuauserv", "methodName":"StartService", "parameters":{}}' | dsc resource set -r 'root.cimv2/Win32_Service' -f -
        $LASTEXITCODE | Should -Be 0
        
        $res = '{"Name":"wuauserv", "State": null}' | dsc resource get -r 'root.cimv2/Win32_Service' -f - | ConvertFrom-Json
        $res.actualState.Name | Should -Be 'wuauserv'
        $res.actualState.State | Should -Be 'Running'

        AfterAll {
            if ($service -and $service.Status -eq 'Running') {
                $service.Stop()
            }
        }
    }
}
