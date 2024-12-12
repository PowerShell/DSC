# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Brew resource tests' {
    BeforeAll {
        $brewExists = ($null -ne (Get-Command brew -CommandType Application -ErrorAction Ignore))
    }

    It 'Config get works' -Skip:(-not $brewExists) {
        $out = dsc config get -f $PSScriptRoot/../examples/brew.dsc.yaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $exists = $null -ne (Get-Command gitui -CommandType Application -ErrorAction Ignore)
        $out.results[1].result.actualState._exist | Should -Be $exists
    }

    It 'Config test works' -Skip:(-not $brewExists) {
        $out = dsc config test -f $PSScriptRoot/../examples/brew.dsc.yaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $exists = $null -ne (Get-Command gitui -CommandType Application -ErrorAction Ignore)
        $out.results[1].result.inDesiredState | Should -Be $exists
    }
}
