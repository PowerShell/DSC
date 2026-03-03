# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for PS adapted manifests' {
    It 'Adapted resource is found' {
        $out = dsc resource list 'PSAdapted*' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.count | Should -Be 2
        $out[0].type | Should -BeExactly 'PSAdaptedTestClassResource/PSAdaptedTestClass'
        $out[0].kind | Should -BeExactly 'resource'
        $out[0].path | Should -Match 'PSAdaptedTestClassResource\.psd1$'
        $out[0].requireAdapter | Should -BeExactly 'Microsoft.Adapter/PowerShell'
        $out[0].schema | Should -Not -BeNullOrEmpty
        $out[1].type | Should -BeExactly 'PSAdaptedTestClassResource/WinPSAdaptedTestClass'
        $out[1].kind | Should -BeExactly 'resource'
        $out[1].path | Should -Match 'PSAdaptedTestClassResource\.psd1$'
        $out[1].requireAdapter | Should -BeExactly 'Microsoft.Adapter/WindowsPowerShell'
        $out[1].schema | Should -Not -BeNullOrEmpty
    }

    It 'Get operation works for adapted resource' {
        $out = dsc resource get -r 'PSAdaptedTestClassResource/PSAdaptedTestClass' -i '{"name":"hello"}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.Name | Should -BeExactly 'hello' -Because ($out | ConvertTo-Json)
        $out.actualState.Value | Should -Be 42
    }

    It 'WinPS adapted resource is not supported' -Skip:(!$IsWindows) {
        $null = dsc resource get -r 'PSAdaptedTestClassResource/WinPSAdaptedTestClass' -i '{"name":"world"}' 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        $errorContent = Get-Content -Path "$TestDrive\error.log" -Raw
        $errorContent | Should -Match 'Adapted resource manifests are not supported on Windows PowerShell.'
    }
}
