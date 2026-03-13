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

    BeforeAll {
        # Use dism command to get a known feature name
        $dismOutput = & dism /Online /Get-Features /Format:Table /English 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to get features using dism: $dismOutput"
        }
        $enabledMatches = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Enabled\s*$'
        $disabledMatches = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Disabled\s*$'
        if (-not $enabledMatches -or -not $disabledMatches) {
            throw "Failed to find both enabled and disabled features in DISM output.`nOutput:`n$dismOutput"
        }
        $knownFeatureNameOne = $enabledMatches[0].Matches[0].Groups[1].Value
        $knownFeatureNameTwo = $disabledMatches[0].Matches[0].Groups[1].Value        
    }

    It 'gets a known optional feature by name' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"' + $knownFeatureNameOne + '"}]}'
        $output = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.features | Should -Not -BeNullOrEmpty
        $output.actualState.features.Count | Should -Be 1
        $feature = $output.actualState.features[0]
        $feature.featureName | Should -BeExactly $knownFeatureNameOne
        $feature.state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
        $feature.displayName | Should -Not -BeNullOrEmpty
        $feature.description | Should -Not -BeNullOrEmpty
        $feature.restartRequired | Should -BeIn @('No', 'Possible', 'Required')
    }

    It 'gets multiple features in a single request' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"' + $knownFeatureNameOne + '"} ,{"featureName":"' + $knownFeatureNameTwo + '"}]}'
        $output = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.features.Count | Should -Be 2
        $output.actualState.features[0].featureName | Should -BeExactly $knownFeatureNameOne
        $output.actualState.features[1].featureName | Should -BeExactly $knownFeatureNameTwo
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

    It 'returns _exist false for a non-existent feature name' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"NonExistent-Feature-1234567890"}]}'
        $output = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.features | Should -Not -BeNullOrEmpty
        $output.actualState.features.Count | Should -Be 1
        $feature = $output.actualState.features[0]
        $feature.featureName | Should -BeExactly 'NonExistent-Feature-1234567890'
        $feature._exist | Should -BeFalse
        $feature.state | Should -BeNullOrEmpty
        $feature.displayName | Should -BeNullOrEmpty
        $feature.description | Should -BeNullOrEmpty
        $feature.restartRequired | Should -BeNullOrEmpty
    }

    It 'returns _exist false alongside valid features' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"' + $knownFeatureNameOne + '"},{"featureName":"NonExistent-Feature-1234567890"}]}'
        $output = dsc resource get -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.features.Count | Should -Be 2
        $output.actualState.features[0].featureName | Should -BeExactly $knownFeatureNameOne
        $output.actualState.features[0].PSObject.Properties.Name | Should -Not -Contain '_exist'
        $output.actualState.features[1].featureName | Should -BeExactly 'NonExistent-Feature-1234567890'
        $output.actualState.features[1]._exist | Should -BeFalse
    }
}
