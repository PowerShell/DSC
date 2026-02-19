# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Import extension tests' {
    It 'Deprecated extension shows message' {
        Set-Content -Path "$TestDrive/test.testimport" -Value 'Test content'
        $null = dsc config get -f "$TestDrive/test.testimport" 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content -Path "$TestDrive/error.log" -Raw | Out-String)
        (Get-Content -Path "$TestDrive/error.log" -Raw) | Should -Match "Extension 'Test/ExtensionDeprecated' is deprecated: This extension is deprecated" -Because (Get-Content -Path "$TestDrive/error.log" -Raw | Out-String)
    }
}
