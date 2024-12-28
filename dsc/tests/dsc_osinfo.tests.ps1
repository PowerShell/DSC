Describe 'Tests for osinfo examples' {
    It 'Config with default parameters and get works' {
        $out = dsc config get -f $PSScriptRoot/../examples/osinfo_parameters.dsc.yaml | ConvertFrom-Json -Depth 10
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
        $out = dsc config -f $PSScriptRoot/../examples/osinfo.parameters.yaml test -f $PSScriptRoot/../examples/osinfo_parameters.dsc.yaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.inDesiredState | Should -Be $IsMacOS
    }

    It 'Verify dsc home directory is added to PATH to find included resources' {
        $oldPath = $env:PATH
        $oldLocation = Get-Location
        try {
            $exe_path = (Get-Command dsc).Path | Split-Path
            $exe_path | Split-Path | Set-Location
            # Remove exe_path from PATH if it is there
            $new_path = ($oldPath.Split([System.IO.Path]::PathSeparator) | Where-Object { $_ -ne $exe_path }) -join [System.IO.Path]::PathSeparator
            $env:PATH = $new_path

            $null = & "$exe_path/dsc" config test -f "$PSScriptRoot/../examples/osinfo_parameters.dsc.yaml"
            $LASTEXITCODE | Should -Be 0
        }
        finally {
            $env:PATH = $oldPath
            $oldLocation | Set-Location
        }
    }
}
