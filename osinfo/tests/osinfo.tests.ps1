# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'osinfo resource tests' {
    It 'should get osinfo' {
        $out = dsc resource get -r Microsoft/osinfo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($IsWindows) {
            $out.actualState.family | Should -BeExactly 'Windows'
        }
        elseif ($IsLinux) {
            $out.actualState.family | Should -BeExactly 'Linux'
        }
        elseif ($IsMacOS) {
            $out.actualState.family | Should -BeExactly 'MacOS'
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
        $out = '{"family": "does_not_exist"}' | dsc resource test -r '*osinfo' | ConvertFrom-Json
        $actual = dsc resource get -r Microsoft/OSInfo | ConvertFrom-Json
        $out.actualState.family | Should -BeExactly $actual.actualState.family
        $out.actualState.version | Should -BeExactly $actual.actualState.version
        $out.actualState.bitness | Should -BeExactly $actual.actualState.bitness
        $out.actualState.edition | Should -BeExactly $actual.actualState.edition
        $out.differingproperties | Should -Be @('family')
    }
}
