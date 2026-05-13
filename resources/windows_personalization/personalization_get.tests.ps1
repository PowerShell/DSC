# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Personalization get tests' {
    It 'Can get a personalization setting' -Skip:(!$IsWindows) {
        $json = @{
            "appUseLightTheme" = $true
        } | ConvertTo-Json -Compress
        $out = dsc resource get -r Microsoft.Windows/Personalization -i $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because "dsc resource get failed with exit code $LASTEXITCODE. Error log: $(Get-Content $TestDrive/error.log -Raw)"
        Write-Verbose -Verbose "Output from dsc resource get: $($out | ConvertTo-Json -Compress)"
        $out.appUseLightTheme | Should -Be $true
    }
}
