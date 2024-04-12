# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'InTune Custom Compliance resource tests' {

    BeforeAll {
    }
    AfterAll {
    }

    It 'Discovery includes the resource' -Skip:(!$IsWindows) {

        $r = dsc resource list
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | Where-Object { $_.Type -eq 'Microsoft.Intune/WindowsCustomCompliance' }).Count | Should -Be 1
    }

    It 'Get works as expected' -Skip:(!$IsWindows) {

        $path = Resolve-Path $PSScriptRoot
        $r = '{"JSON":"' + $path.path.replace('\', '\\') + '\\example.json", "script":"' + $path.path.replace('\', '\\') + '\\example.ps1"}' | dsc resource get -r 'Microsoft.Intune/WindowsCustomCompliance'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState | Where-Object SettingName -EQ 'Manufacturer' | % compliance | Should -BeExactly 'compliant'
    }
}
