Describe 'Tests for osinfo examples' {
    It 'Config with default parameters and get works' {
        $out = dsc config get -p $PSScriptRoot/../examples/osinfo_parameters.dsc.yaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $expected = if ($IsWindows) {
            'Windows'
        } elseif ($IsLinux) {
            'Linux'
        } else {
            'macOS'
        }

        $out.results[0].result.actualState.family | Should -BeExactly $expected
    }

    It 'Config test works' {
        $out = dsc config -f $PSScriptRoot/../examples/osinfo.parameters.yaml test -p $PSScriptRoot/../examples/osinfo_parameters.dsc.yaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.inDesiredState | Should -Be $IsMacOS
    }
}
