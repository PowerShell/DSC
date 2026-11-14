# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/WindowsFeatureList - get operation' -Skip:(!$IsWindows) {
    BeforeAll {
        # Discover at least one enabled and one disabled feature using DISM
        $dismOutput = & dism /Online /Get-Features /Format:Table /English 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to enumerate features using dism: $dismOutput"
        }
        $enabledMatches  = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Enabled\s*$'
        $disabledMatches = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Disabled\s*$'
        if (-not $enabledMatches -or -not $disabledMatches) {
            throw "Failed to find both enabled and disabled features in DISM output.`nOutput:`n$dismOutput"
        }
        $knownEnabledFeature  = $enabledMatches[0].Matches[0].Groups[1].Value
        $knownDisabledFeature = $disabledMatches[0].Matches[0].Groups[1].Value
    }

    Context 'Get a single feature by featureName' {
        It 'returns feature info for a known enabled feature' {
            $inputJson = '{"features":[{"featureName":"' + $knownEnabledFeature + '"}]}'
            $output = dsc resource get -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $output.actualState.features | Should -Not -BeNullOrEmpty
            $output.actualState.features.Count | Should -Be 1
            $feature = $output.actualState.features[0]
            $feature.featureName | Should -BeExactly $knownEnabledFeature
            $feature.state | Should -BeIn @(
                'NotPresent', 'UninstallPending', 'Staged', 'Removed',
                'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
            )
            $feature.displayName | Should -Not -BeNullOrEmpty
            $feature.description | Should -Not -BeNullOrEmpty
            $feature.restartRequired | Should -BeIn @('No', 'Possible', 'Required')
        }

        It 'returns feature info for a known disabled feature' {
            $inputJson = '{"features":[{"featureName":"' + $knownDisabledFeature + '"}]}'
            $output = dsc resource get -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $feature = $output.actualState.features[0]
            $feature.featureName | Should -BeExactly $knownDisabledFeature
            $feature.state | Should -BeIn @(
                'NotPresent', 'UninstallPending', 'Staged', 'Removed',
                'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
            )
        }

        It 'returns _exist false for a non-existent feature name' {
            $inputJson = '{"features":[{"featureName":"NonExistent-Feature-1234567890"}]}'
            $output = dsc resource get -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $feature = $output.actualState.features[0]
            $feature.featureName | Should -BeExactly 'NonExistent-Feature-1234567890'
            $feature._exist | Should -BeFalse
            $feature.PSObject.Properties.Name | Should -Not -Contain 'state'
            $feature.PSObject.Properties.Name | Should -Not -Contain 'displayName'
        }
    }

    Context 'Get multiple features in one request' {
        It 'returns info for both features' {
            $inputJson = '{"features":[{"featureName":"' + $knownEnabledFeature + '"},{"featureName":"' + $knownDisabledFeature + '"}]}'
            $output = dsc resource get -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $output.actualState.features.Count | Should -Be 2
            $output.actualState.features[0].featureName | Should -BeExactly $knownEnabledFeature
            $output.actualState.features[1].featureName | Should -BeExactly $knownDisabledFeature
        }
    }

    Context 'Input validation' {
        It 'returns error when featureName is missing' {
            $inputJson = '{"features":[{"state":"Installed"}]}'
            & { dsc resource get -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>&1 } | Out-Null
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'returns error when features array is empty' {
            $inputJson = '{"features":[]}'
            & { dsc resource get -r Microsoft.Windows/WindowsFeatureList -i $inputJson 2>&1 } | Out-Null
            $LASTEXITCODE | Should -Not -Be 0
        }
    }
}
