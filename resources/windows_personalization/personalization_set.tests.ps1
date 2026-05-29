# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Personalization resource set tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $currentSettings = dsc resource export -r Microsoft.Windows/Personalization 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because "Failed to export current personalization settings with exit code $LASTEXITCODE. Error log: $(Get-Content $TestDrive/error.log -Raw)"
    }

    AfterAll {
        dsc resource set -r Microsoft.Windows/Personalization -i $currentSettings 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    }

    It 'Can set personalization settings' {
        $newSettings = @{
            appsUseLightTheme = $true
            systemUsesLightTheme = $true
            autoColorization = $true
            colorPrevalence = $true
            transparencyEffects = $true
            startMenuVisiblePlaces = @('Documents', 'Downloads','Music', 'Pictures', 'Videos', 'Network', 'UserProfile', 'Explorer', 'Settings')
            startMenuShowRecentList = $true
            showRecommendedApps = $true
            taskbarShowBadges = $true
            desktopTaskbarShowBadges = $true
            multimonitorTaskbarGroupingMode = 'CombineWhenTaskbarIsFull'
            multimonitorTaskbar = $true
            multimonitorDesktopTaskbar = $true
            multimonitorTaskbarMode = 'PrimaryAndMonitorWindowIsOn'
            multimonitorDesktopTaskbarMode = 'MonitorWindowIsOn'
        }

        dsc resource set -r Microsoft.Windows/Personalization -i ($newSettings | ConvertTo-Json) 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)

        $out = dsc resource export -r Microsoft.Windows/Personalization 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        foreach ($key in $newSettings.Keys) {
            $keyValue = $out.resources[0].properties.$key
            $expectedValue = $newSettings.$key
            $keyValue | Should -Be $expectedValue -Because "Property $key has value $keyValue which is not equal to expected value $expectedValue"
        }
    }
}
