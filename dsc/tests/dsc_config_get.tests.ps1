Describe 'dsc config get tests' {
    It 'can successfully get config with multiple registry resource instances' -Skip:(!$IsWindows) {
        $jsonPath = Join-Path $PSScriptRoot 'osinfo_registry.dsc.json'
        $config = Get-Content $jsonPath -Raw
        $out = $config | dsc config get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.hadErrors | Should -BeFalse
        $out.results.Count | Should -Be 3
        $out.results[0].Name | Should -Be 'os'
        $out.results[0].type | Should -Be 'osinfo'
        $out.results[0].result.actual_state.family | Should -BeExactly 'Windows'
        $out.results[1].Name | Should -Be 'windows product name'
        $out.results[1].type | Should -Be 'registry'
        $out.results[1].result.actual_state.valueData.String | Should -BeLike 'Windows*'
        $out.results[2].Name | Should -Be 'system root'
        $out.results[2].type | Should -Be 'registry'
        $out.results[2].result.actual_state.valueData.String | Should -BeExactly $env:SystemRoot
    }

    It 'will fail if resource schema does not match' -Skip:(!$IsWindows) {
        $jsonPath = Join-Path $PSScriptRoot 'invalid_schema.dsc.json'
        $config = Get-Content $jsonPath -Raw
        $out = $config | dsc config get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2
        $out.hadErrors | Should -BeTrue
        $out.results.Count | Should -Be 0
        $out.messages.Count | Should -Be 3
        $out.messages[0].level | Should -BeExactly 'Warning'
        $out.messages[1].level | Should -BeExactly 'Error'
        $out.messages[2].level | Should -BeExactly 'Error'
    }
}
