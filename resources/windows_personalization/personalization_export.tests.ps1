# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Personalization resource export tests' -Skip:(!$IsWindows) {
    It 'Export works with no input' {
        $properties = @{
            appsUseLightTheme = @($true, $false)
            systemUsesLightTheme = @($true, $false)
            autoColorization = @($true, $false)
            colorPrevalence = @($true, $false)
            transparencyEffects = @($true, $false)
            startMenuVisiblePlaces = @('Documents', 'Downloads','Music', 'Pictures', 'Videos', 'Network', 'UserProfile', 'Explorer', 'Settings')
            startMenuShowRecentList = @($true, $false)
            showRecommendedApps = @($true, $false)
            taskbarShowBadges = @($true, $false)
            desktopTaskbarShowBadges = @($true, $false)
            multimonitorTaskbarGroupingMode = @('AlwaysCombineHideLabels', 'CombineWhenTaskbarIsFull', 'NeverCombine')
            multimonitorTaskbar = @($true, $false)
            multimonitorDesktopTaskbar = @($true, $false)
            multimonitorTaskbarMode = @('Duplicate', 'PrimaryAndMonitorWindowIsOn', 'MonitorWindowIsOn')
            multimonitorDesktopTaskbarMode = @('Duplicate', 'PrimaryAndMonitorWindowIsOn', 'MonitorWindowIsOn')
        }
        $out = dsc resource export -r Microsoft.Windows/Personalization 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        $resultProperties = $out.resources[0].properties.psobject.properties
        ($resultProperties | Measure-Object).Count | Should -Be $properties.Count
        foreach ($key in $properties.Keys) {
            $resultProperties[$key].Value | Should -BeIn $properties[$key] -Because "Property $key has value $($resultProperties[$key].Value) which is not in expected values $($properties[$key])"
        }
    }
}
