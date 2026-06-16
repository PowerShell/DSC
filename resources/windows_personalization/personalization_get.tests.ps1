# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Personalization get tests' {
    It 'Convert dword to boolean' -Skip:(!$IsWindows) {
        $json = @{
            "appsUseLightTheme" = $true
        } | ConvertTo-Json -Compress
        $out = dsc resource get -r Microsoft.Windows/Personalization -i $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because "dsc resource get failed with exit code $LASTEXITCODE. Error log: $(Get-Content $TestDrive/error.log -Raw)"
        $out.actualState.appsUseLightTheme | Should -BeIn @($true, $false)
    }

    It 'Convert binary to string array' -Skip:(!$IsWindows) {
        $json = @{
            "startMenuVisiblePlaces" = @()
        } | ConvertTo-Json -Compress

        $existingVisiblePlaces = (Get-ItemProperty -path HKCU:\Software\Microsoft\Windows\CurrentVersion\Start -Name VisiblePlaces).VisiblePlaces

        try {
            $allVisiblePlaces = @(134, 8, 115, 82, 170, 81, 67, 66, 159, 123, 39, 118, 88, 70, 89, 212, 206, 213, 52, 45, 90, 250, 67, 69, 130, 242,
                34, 230, 234, 247, 119, 60, 47, 179, 103, 227, 222, 137, 85, 67, 191, 206, 97, 243, 123, 24, 169, 55, 32, 6, 11, 176, 81, 127, 50, 76,
                170, 30, 52, 204, 84, 127, 115, 21, 160, 7, 63, 56, 10, 232, 128, 76, 176, 90, 134, 219, 132, 93, 188, 77, 197, 165, 179, 66, 134, 125,
                244, 66, 128, 164, 147, 250, 202, 122, 136, 181, 68, 129, 117, 254, 13, 8, 174, 66, 139, 218, 52, 237, 151, 182, 99, 148, 74, 176, 189,
                116, 74, 249, 104, 79, 139, 214, 67, 152, 7, 29, 168, 188, 188, 36, 138, 20, 12, 214, 137, 66, 160, 128, 110, 217, 187, 162, 72, 130)
            Set-ItemProperty -path HKCU:\Software\Microsoft\Windows\CurrentVersion\Start -Name VisiblePlaces -Value $allVisiblePlaces
            $out = dsc resource get -r Microsoft.Windows/Personalization -i $json 2>$TestDrive/error.log | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because "dsc resource get failed with exit code $LASTEXITCODE. Error log: $(Get-Content $TestDrive/error.log -Raw)"
            $out.actualState.startMenuVisiblePlaces | Should -BeExactly @("Settings","Documents","Downloads","Music","Pictures","Videos","Network","UserProfile","Explorer")
        } finally {
            Set-ItemProperty -path HKCU:\Software\Microsoft\Windows\CurrentVersion\Start -Name VisiblePlaces -Value $existingVisiblePlaces
        }
    }

    It 'Convert dword to string enum' -Skip:(!$IsWindows) {
        $json = @{
            "multimonitorTaskbarGroupingMode" = 'NeverCombine'
        } | ConvertTo-Json -Compress
        $out = dsc resource get -r Microsoft.Windows/Personalization -i $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because "dsc resource get failed with exit code $LASTEXITCODE. Error log: $(Get-Content $TestDrive/error.log -Raw)"
        $out.actualState.multimonitorTaskbarGroupingMode | Should -BeIn @('NeverCombine', 'AlwaysCombineHideLabels', 'CombineWhenTaskbarIsFull')
    }
}
