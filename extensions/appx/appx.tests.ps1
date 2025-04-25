# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for Appx resource discovery' {
    BeforeAll {
        $skip = (!$IsWindows -or ($null -eq (get-appxpackage -Name Microsoft.DesiredStateConfiguration-Preview)))
    }

    It 'Should find DSC appx resources' -Skip:$skip {
        $out = dsc resource list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $found = $false
        foreach ($resource in $out) {
            if ($resource.directory.StartsWith("$env:ProgramFiles\WindowsApps\Microsoft.DesiredStateConfiguration-Private")) {
                $found = $true
                break
            }
        }
    }
}
