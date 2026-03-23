# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for PS adapted manifests' {
    It 'Adapted resource is found' {
        $out = dsc resource list 'PSAdapted*' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.count | Should -Be 1
        $out.type | Should -BeExactly 'PSAdaptedTestClassResource/PSAdaptedTestClass'
        $out.kind | Should -BeExactly 'resource'
        $out.path | Should -Match 'PSAdaptedTestClassResource\.psd1$'
        $out.requireAdapter | Should -BeExactly 'Microsoft.Adapter/PowerShell'
        $out.schema | Should -Not -BeNullOrEmpty
    }

    It 'Get operation works for adapted resource' {
        $out = dsc resource get -r 'PSAdaptedTestClassResource/PSAdaptedTestClass' -i '{"name":"hello"}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.Name | Should -BeExactly 'hello' -Because ($out | ConvertTo-Json)
        $out.actualState.Value | Should -Be 42
    }
}
