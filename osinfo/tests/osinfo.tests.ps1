Describe 'osinfo resource tests' {
    It 'should get osinfo' {
        $out = dsc resource get -r osinfo
        $LASTEXITCODE | Should -Be 0
        $out = $out | ConvertFrom-Json
        if ($IsWindows) {
            $out.actual_state.type | Should -BeExactly 'Windows'
        }
        elseif ($IsLinux) {
            $out.actual_state.type | Should -BeExactly 'Linux'
        }
        elseif ($IsMacOS) {
            $out.actual_state.type | Should -BeExactly 'MacOS'
        }

        $out.actual_state.version | Should -Not -BeNullOrEmpty
        if ([Environment]::Is64BitProcess) {
            $out.actual_state.bitness | Should -BeExactly 'X64'
        }
        else {
            $out.actual_state.bitness | Should -BeExactly 'X32'
        }
    }
}
