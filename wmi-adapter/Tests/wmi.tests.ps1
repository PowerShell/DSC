# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WMI adapter resource tests' {

    BeforeAll {
        if ($IsWindows)
        {
            $OldPSModulePath  = $env:PSModulePath
            $env:PSModulePath += ";" + $PSScriptRoot

            $configPath = Join-path $PSScriptRoot "test_wmi_config.dsc.yaml"

            $dscPath = (get-command dsc -CommandType Application | Select-Object -First 1).Path
            $dscFolder = Split-Path -Path $dscPath
            $wmiGroupOptoutFile = Join-Path $dscFolder "wmi.dsc.resource.json.optout"
            $wmiGroupOptinFile = Join-Path $dscFolder "wmi.dsc.resource.json"
            Rename-Item -Path $wmiGroupOptoutFile -NewName $wmiGroupOptinFile
        }
    }
    AfterAll {
        if ($IsWindows)
        {
            $env:PSModulePath = $OldPSModulePath
            Rename-Item -Path $wmiGroupOptinFile -NewName $wmiGroupOptoutFile
        }
    }

    It 'List shows WMI resources' -Skip:(!$IsWindows){

        $r = dsc resource list *OperatingSystem*
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

        $r = Get-Content -Raw $configPath | dsc config get
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState[0].LastBootUpTime | Should -Not -BeNull
        $res.results[0].result.actualState[1].BiosCharacteristics | Should -Not -BeNull
        $res.results[0].result.actualState[2].NumberOfLogicalProcessors | Should -Not -BeNull
    }
}
