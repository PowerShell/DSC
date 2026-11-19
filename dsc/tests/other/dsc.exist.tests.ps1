# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe '_exist tests' {
    It 'Resource supporting exist on set should receive _exist for: <exist>' -TestCases @(
        @{ exist = $true }
        @{ exist = $false }
    ) {
        param($exist)

        $json = @"
        {
            "_exist": $exist
        }
"@
        $out = dsc resource set -r Test/Exist --input $json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $out.afterState._exist | Should -Be $exist
        if ($exist) {
            $out.afterState.state | Should -Be 'Present'
        }
        else {
            $out.afterState.state | Should -Be 'Absent'
        }
    }
}
