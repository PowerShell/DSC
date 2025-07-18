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

    It 'Set does not work without input for resource' -Skip:(!$IsWindows) {
        $s = dsc resource set --resource root.cimv2/Win32_Environment --input '{}' 2>&1
        $LASTEXITCODE | Should -Be 1
        $s | Should -BeLike "*No valid properties found in the CIM class 'Win32_Environment' for the provided properties.*"
    }

    It 'Set does not work without a key property' -Skip:(!$IsWindows) {
        $i = @{
            VariableValue = "TestValue"
            UserName = ("{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME) # Read-only property is key, but we require a key property to be set
        } | ConvertTo-Json
        
        $s = dsc resource set -r root.cimv2/Win32_Environment -i $i 2>&1
        $LASTEXITCODE | Should -Be 1
        $s | Should -BeLike "*All key properties in the CIM class 'Win32_Environment' are read-only, which is not supported.*"
    }

    It 'Set works on a WMI resource' -Skip:(!$IsWindows) {
        $i = @{
            UserName = ("{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME) # Read-only key property required
            Name = 'test'
            VariableValue = 'test'
        } | ConvertTo-Json
        
        $r = dsc resource set -r root.cimv2/Win32_Environment -i $i
        $LASTEXITCODE | Should -Be 0 

        $out = $r | ConvertFrom-Json
        $out.afterState.Name | Should -Be 'test'
        $out.afterState.VariableValue | Should -Be 'test'
        $out.afterState.UserName | Should -Be ("{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME)
    }

    It 'Update works on a WMI resource' -Skip:(!$IsWindows) {
        $i = @{
            UserName = ("{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME) # Read-only key property required
            Name = 'test'
            VariableValue = 'update'
        } | ConvertTo-Json
        
        $r = dsc resource set -r root.cimv2/Win32_Environment -i $i
        $LASTEXITCODE | Should -Be 0 

        $out = $r | ConvertFrom-Json
        $out.afterState.Name | Should -Be 'test'
        $out.afterState.VariableValue | Should -Be 'update'
        $out.afterState.UserName | Should -Be ("{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME)
    }
}
