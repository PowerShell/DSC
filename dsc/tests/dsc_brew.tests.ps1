# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Brew resource tests' {
    BeforeAll {
        $originalDefaultParameterValues = $PSDefaultParameterValues.Clone()
        $PSDefaultParameterValues['it:skip'] = ($null -eq (Get-Command brew -CommandType Application -ErrorAction Ignore))
    }

    AfterAll {
        $global:PSDefaultParameterValues = $originalDefaultParameterValues
    }

    It 'Config get works' {
        $out = dsc config get -p $PSScriptRoot/../examples/brew.dsc.yaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $exists = $null -ne (Get-Command gitui -CommandType Application -ErrorAction Ignore)
        $out.results[1].result.actualState._exist | Should -Be $exists
    }

    It 'Config test works' {
        $out = dsc config test -p $PSScriptRoot/../examples/brew.dsc.yaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $exists = $null -ne (Get-Command gitui -CommandType Application -ErrorAction Ignore)
        $out.results[1].result.inDesiredState | Should -Be $exists
    }
}
