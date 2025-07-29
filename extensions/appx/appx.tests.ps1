# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for Appx resource discovery' -Skip:(!$IsWindows){
    It 'Should find DSC appx resources' {
        $out = dsc resource list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $found = $false
        foreach ($resource in $out) {
            if ($resource.directory.StartsWith("$env:ProgramFiles\WindowsApps")) {
                $found = $true
                break
            }
        }
        $found | Should -Be $true
    }
}
