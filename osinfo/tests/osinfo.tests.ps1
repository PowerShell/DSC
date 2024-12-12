# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'osinfo resource tests' {
    It 'should get osinfo' {
        $out = dsc resource get -r Microsoft/osInfo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($IsWindows) {
            $out.actualState.family | Should -BeExactly 'Windows'
        }
        elseif ($IsLinux) {
            $out.actualState.family | Should -BeExactly 'Linux'
        }
        elseif ($IsMacOS) {
            $out.actualState.family | Should -BeExactly 'macOS'
        }

        $out.actualState.version | Should -Not -BeNullOrEmpty
        if ([Environment]::Is64BitProcess) {
            $out.actualState.bitness | Should -BeExactly '64'
        }
        else {
            $out.actualState.bitness | Should -BeExactly '32'
        }
    }

    It 'should perform synthetic test' {
        if ($IsWindows) {
            $invalid = 'Linux'
        }
        else {
            $invalid = 'Windows'
        }
        $out = "{`"family`": `"$invalid`"}" | dsc resource test -r 'Microsoft/OSInfo' -f - | ConvertFrom-Json
        $actual = dsc resource get -r Microsoft/OSInfo | ConvertFrom-Json
        $out.actualState.family | Should -BeExactly $actual.actualState.family
        $out.actualState.version | Should -BeExactly $actual.actualState.version
        $out.actualState.bitness | Should -BeExactly $actual.actualState.bitness
        $out.actualState.edition | Should -BeExactly $actual.actualState.edition
        $out.differingproperties | Should -Be @('family')
    }

    It 'should support export' {
        $out = dsc resource export -r Microsoft/OSInfo | ConvertFrom-Json
        $out.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
        if ($IsWindows) {
            $out.resources[0].properties.family | Should -BeExactly 'Windows'
        }
        elseif ($IsLinux) {
            $out.resources[0].properties.family | Should -BeExactly 'Linux'
        }
        elseif ($IsMacOS) {
            $out.resources[0].properties.family | Should -BeExactly 'macOS'
        }
    }
}
