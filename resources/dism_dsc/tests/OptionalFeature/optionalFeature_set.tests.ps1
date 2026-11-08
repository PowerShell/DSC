# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/OptionalFeatureList - set operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    It 'returns error when featureName is missing' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"state":"Installed"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error when state is missing' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"Printing-PrintToPDFServices-Features"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error when features array is empty' -Skip:(!$isElevated) {
        $inputJson = '{"features":[]}'
        $testError = & { dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error for unsupported desired state' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"Printing-PrintToPDFServices-Features","state":"Staged"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error for a non-existent feature name' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"NonExistent-Feature-1234567890","state":"Installed"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'can enable a feature and returns updated state' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"Printing-PrintToPDFServices-Features","state":"Installed"}]}'
        $output = dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.features | Should -Not -BeNullOrEmpty
        $output.afterState.features.Count | Should -Be 1
        $feature = $output.afterState.features[0]
        $feature.featureName | Should -BeExactly 'Printing-PrintToPDFServices-Features'
        $feature.state | Should -BeIn @('Installed', 'InstallPending')
        $feature.displayName | Should -Not -BeNullOrEmpty
    }

    It 'enables required parent features automatically' -Skip:(!$isElevated) {
        $getJson = '{"features":[{"featureName":"IIS-WebServer"},{"featureName":"IIS-WebServerRole"}]}'
        $before = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $getJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if (@($before.actualState.features | Where-Object { $_.state -ne 'NotPresent' }).Count -gt 0) {
            Set-ItResult -Skipped -Because 'The IIS feature and its parent must initially be disabled.'
            return
        }

        try {
            $inputJson = '{"features":[{"featureName":"IIS-WebServer","state":"Installed"}]}'
            $output = dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $output.afterState.features.Count | Should -Be 1
            $output.afterState.features[0].featureName | Should -BeExactly 'IIS-WebServer'
            $output.afterState.features[0].state | Should -BeIn @('Installed', 'InstallPending')

            $after = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $getJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $parent = $after.actualState.features | Where-Object { $_.featureName -eq 'IIS-WebServerRole' }
            $parent.state | Should -BeIn @('Installed', 'InstallPending')
        } finally {
            $restoreJson = '{"features":[{"featureName":"IIS-WebServer","state":"NotPresent"},{"featureName":"IIS-WebServerRole","state":"NotPresent"}]}'
            dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $restoreJson | Out-Null
            $LASTEXITCODE | Should -Be 0
        }
    }

    It 'can disable a feature with NotPresent and returns updated state' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"Printing-PrintToPDFServices-Features","state":"NotPresent"}]}'
        $output = dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.features | Should -Not -BeNullOrEmpty
        $output.afterState.features.Count | Should -Be 1
        $feature = $output.afterState.features[0]
        $feature.featureName | Should -BeExactly 'Printing-PrintToPDFServices-Features'
        $feature.state | Should -BeIn @('NotPresent', 'Removed', 'UninstallPending', 'Staged')
    }

    It 'sets state to Installed for an already installed feature' -Skip:(!$isElevated) {
        # First ensure the feature is installed
        $enableJson = '{"features":[{"featureName":"Printing-PrintToPDFServices-Features","state":"Installed"}]}'
        dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $enableJson | Out-Null
        $LASTEXITCODE | Should -Be 0

        # Now set Installed again — should succeed idempotently
        $output = dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $enableJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.features | Should -Not -BeNullOrEmpty
        $feature = $output.afterState.features[0]
        $feature.featureName | Should -BeExactly 'Printing-PrintToPDFServices-Features'
        $feature.state | Should -Be 'Installed'
    }

    It 'sets state to NotPresent for an already not-present feature' -Skip:(!$isElevated) {
        # First ensure the feature is not present
        $disableJson = '{"features":[{"featureName":"Printing-PrintToPDFServices-Features","state":"NotPresent"}]}'
        dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $disableJson | Out-Null
        $LASTEXITCODE | Should -Be 0

        # Now set NotPresent again — should succeed idempotently
        $output = dsc resource set -r Microsoft.Windows/OptionalFeatureList -i $disableJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.features | Should -Not -BeNullOrEmpty
        $feature = $output.afterState.features[0]
        $feature.featureName | Should -BeExactly 'Printing-PrintToPDFServices-Features'
        $feature.state | Should -BeIn @('NotPresent', 'Removed', 'Staged')
    }
}
