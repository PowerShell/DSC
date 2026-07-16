# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/OptionalFeatureList - export operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    It 'exports all features with no input' -Skip:(!$isElevated) {
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList | ConvertFrom-Json
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

    It 'returns an error when export input is provided' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"TelnetClient"}]}'
        dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>$TESTDRIVE/error.log | Out-Null
        $LASTEXITCODE | Should -Be 2
        (Get-Content -Raw $TESTDRIVE/error.log) | Should -Match 'does not support export filtering'
    }
}
