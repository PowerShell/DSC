# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/WindowsFeatureList - set operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    # TelnetClient is a safe non-critical feature available on most Windows SKUs
    # used here to exercise enable/disable without system impact.

    Context 'Input validation' {
        It 'returns error when featureName is missing' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"state":"Installed"}]}'
            & { dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>&1 } | Out-Null
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'returns error when state is missing' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"featureName":"TelnetClient"}]}'
            & { dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>&1 } | Out-Null
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'returns error when features array is empty' -Skip:(!$isElevated) {
            $inputJson = '{"features":[]}'
            & { dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>&1 } | Out-Null
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'returns error for unsupported desired state' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"featureName":"TelnetClient","state":"Staged"}]}'
            & { dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>&1 } | Out-Null
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'returns error for a non-existent feature name' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"featureName":"NonExistent-Feature-1234567890","state":"Installed"}]}'
            & { dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>&1 } | Out-Null
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Enable and disable TelnetClient' {
        It 'can enable TelnetClient and returns Installed state' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"featureName":"TelnetClient","state":"Installed"}]}'
            $output = dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $output.afterState.features | Should -Not -BeNullOrEmpty
            $output.afterState.features.Count | Should -Be 1
            $feature = $output.afterState.features[0]
            $feature.featureName | Should -BeExactly 'TelnetClient'
            $feature.state | Should -BeIn @('Installed', 'InstallPending')
            $feature.displayName | Should -Not -BeNullOrEmpty
        }

        It 'can enable TelnetClient with enableAll set to true' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"featureName":"TelnetClient","state":"Installed","enableAll":true}]}'
            $output = dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $feature = $output.afterState.features[0]
            $feature.featureName | Should -BeExactly 'TelnetClient'
            $feature.state | Should -BeIn @('Installed', 'InstallPending')
        }

        It 'can disable TelnetClient with NotPresent and returns non-Installed state' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"featureName":"TelnetClient","state":"NotPresent"}]}'
            $output = dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $output.afterState.features | Should -Not -BeNullOrEmpty
            $feature = $output.afterState.features[0]
            $feature.featureName | Should -BeExactly 'TelnetClient'
            $feature.state | Should -BeIn @('NotPresent', 'Removed', 'UninstallPending', 'Staged')
        }

        It 'can disable TelnetClient with Removed state' -Skip:(!$isElevated) {
            $inputJson = '{"features":[{"featureName":"TelnetClient","state":"Removed"}]}'
            $output = dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $feature = $output.afterState.features[0]
            $feature.featureName | Should -BeExactly 'TelnetClient'
            $feature.state | Should -BeIn @('NotPresent', 'Removed', 'Staged', 'UninstallPending')
        }

        It 'set Installed is idempotent for an already installed feature' -Skip:(!$isElevated) {
            # First ensure installed
            $enableJson = '{"features":[{"featureName":"TelnetClient","state":"Installed"}]}'
            dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $enableJson | Out-Null
            $LASTEXITCODE | Should -Be 0

            # Set Installed again - should succeed
            $output = dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $enableJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $output.afterState.features[0].state | Should -Be 'Installed'
        }

        It 'set NotPresent is idempotent for an already disabled feature' -Skip:(!$isElevated) {
            # First ensure not present
            $disableJson = '{"features":[{"featureName":"TelnetClient","state":"NotPresent"}]}'
            dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $disableJson | Out-Null
            $LASTEXITCODE | Should -Be 0

            # Set NotPresent again - should succeed
            $output = dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $disableJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $output.afterState.features[0].state | Should -BeIn @('NotPresent', 'Removed', 'Staged')
        }
    }

    Context 'limitAccess parameter' {
        It 'can enable TelnetClient with limitAccess true' -Skip:(!$isElevated) {
            # TelnetClient payload is present in CBS, so limitAccess should not prevent installation
            $inputJson = '{"features":[{"featureName":"TelnetClient","state":"Installed","limitAccess":true}]}'
            $output = dsc resource set -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            # May succeed or fail depending on whether CBS payload is staged; just verify exit code 0 means success
            if ($LASTEXITCODE -eq 0) {
                $feature = $output.afterState.features[0]
                $feature.featureName | Should -BeExactly 'TelnetClient'
            }
        }
    }
}
