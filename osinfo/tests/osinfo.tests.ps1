# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'osinfo resource tests' {
    It 'should get osinfo' {
        $out = dsc3 resource get -r Microsoft/osinfo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($IsWindows) {
            $out.actual_state.family | Should -BeExactly 'Windows'
        }
        elseif ($IsLinux) {
            $out.actual_state.family | Should -BeExactly 'Linux'
        }
        elseif ($IsMacOS) {
            $out.actual_state.family | Should -BeExactly 'MacOS'
        }

        $out.actual_state.version | Should -Not -BeNullOrEmpty
        if ([Environment]::Is64BitProcess) {
            $out.actual_state.bitness | Should -BeExactly '64'
        }
        else {
            $out.actual_state.bitness | Should -BeExactly '32'
        }
    }

    It 'should perform synthetic test' {
        $out = '{"family": "does_not_exist"}' | dsc3 resource test -r '*osinfo' | ConvertFrom-Json
        $actual = dsc3 resource get -r Microsoft/OSInfo | ConvertFrom-Json
        $out.actual_state.family | Should -BeExactly $actual.actual_state.family
        $out.actual_state.version | Should -BeExactly $actual.actual_state.version
        $out.actual_state.bitness | Should -BeExactly $actual.actual_state.bitness
        $out.actual_state.edition | Should -BeExactly $actual.actual_state.edition
        $out.diff_properties | Should -Be @('family')
    }
}
