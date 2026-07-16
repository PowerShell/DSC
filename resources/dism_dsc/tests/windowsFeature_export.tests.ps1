# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/WindowsFeatureList - export operation' -Skip:(!$IsWindows) {
    It 'exports all features with no input filter' {
        $output = dsc resource export -r Microsoft.Windows/WindowsFeatureList | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        $features.Count | Should -BeGreaterThan 0
        $features[0].featureName | Should -Not -BeNullOrEmpty
        $features[0].state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
    }

    It 'returns an error when export input is provided' {
        $inputJson = '{"features":[{"featureName":"Web-Server"}]}'
        dsc resource export -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>$TESTDRIVE/error.log | Out-Null
        $LASTEXITCODE | Should -Be 2
        (Get-Content -Raw $TESTDRIVE/error.log) | Should -Match 'does not support export filtering'
    }
}
