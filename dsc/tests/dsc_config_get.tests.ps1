Describe 'dsc config get tests' {
    It 'can successfully get config with multiple registry resource instances' -Skip:(!$IsWindows) {
        $jsonPath = Join-Path $PSScriptRoot 'osinfo_registry.dsc.json'
        $config = Get-Content $jsonPath -Raw
        $out = $config | dsc config get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.Count | Should -Be 3
        $out.results[0].Name | Should -Be 'os'
        $out.results[0].type | Should -Be 'osinfo'
        $out.results[0].result.actual_state.family | Should -BeExactly 'Windows'
        $out.results[1].Name | Should -Be 'windows product name'
        $out.results[1].type | Should -Be 'registry'
        $out.results[1].result.actual_state.valueData.String | Should -BeLike 'Windows*'
        $out.results[2].Name | Should -Be 'powershell version'
        $out.results[2].type | Should -Be 'registry'
        $out.results[2].result.actual_state.valueData.String | Should -BeLike '7.*'
    }
}
