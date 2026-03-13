# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/OptionalFeatureList - get operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    It 'gets a known optional feature by name' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"Microsoft-Windows-Subsystem-Linux"}]}'
        $output = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.features | Should -Not -BeNullOrEmpty
        $output.actualState.features.Count | Should -Be 1
        $feature = $output.actualState.features[0]
        $feature.featureName | Should -BeExactly 'Microsoft-Windows-Subsystem-Linux'
        $feature.state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
        $feature.displayName | Should -Not -BeNullOrEmpty
        $feature.description | Should -Not -BeNullOrEmpty
        $feature.restartRequired | Should -BeIn @('No', 'Possible', 'Required')
    }

    It 'gets multiple features in a single request' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"Microsoft-Windows-Subsystem-Linux"},{"featureName":"Printing-PrintToPDFServices-Features"}]}'
        $output = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.features.Count | Should -Be 2
        $output.actualState.features[0].featureName | Should -BeExactly 'Microsoft-Windows-Subsystem-Linux'
        $output.actualState.features[1].featureName | Should -BeExactly 'Printing-PrintToPDFServices-Features'
    }

    It 'returns error when featureName is missing' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"state":"Installed"}]}'
        $testError = & { dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error when features array is empty' -Skip:(!$isElevated) {
        $inputJson = '{"features":[]}'
        $testError = & { dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error for a non-existent feature name' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"NonExistent-Feature-1234567890"}]}'
        $testError = & { dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }
}
