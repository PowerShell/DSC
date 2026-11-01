# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/WindowsFeatureList - export operation' -Skip:(!$IsWindows) {
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

    It 'exports features filtered by exact featureName' {
        $inputJson = '{"features":[{"featureName":"' + $knownEnabledFeature + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        $features.Count | Should -Be 1
        $features[0].featureName | Should -BeExactly $knownEnabledFeature
    }

    It 'exports features filtered by state Installed' {
        $inputJson = '{"features":[{"state":"Installed"}]}'
        $output = dsc resource export -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        $features | ForEach-Object { $_.state | Should -Be 'Installed' }
    }

    It 'returns empty features list for a non-matching filter' {
        $inputJson = '{"features":[{"featureName":"NonExistent-Feature-1234567890"}]}'
        $output = dsc resource export -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -BeNullOrEmpty
    }

    It 'exports with wildcard featureName filter' {
        # Use the first 3 characters of a known feature name as a wildcard prefix
        $prefix = $knownEnabledFeature.Substring(0, [Math]::Min(3, $knownEnabledFeature.Length))
        $inputJson = '{"features":[{"featureName":"' + $prefix + '*"}]}'
        $output = dsc resource export -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        # At minimum the known feature should be present if its name starts with $prefix
        $features | Should -Not -BeNullOrEmpty
        $features | ForEach-Object {
            $_.featureName.ToLower() | Should -BeLike "$($prefix.ToLower())*"
        }
    }

    It 'exports multiple feature filters (OR logic)' {
        $inputJson = '{"features":[{"featureName":"' + $knownEnabledFeature + '"},{"featureName":"' + $knownDisabledFeature + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/WindowsFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $featureNames = $output.resources[0].properties.features | Select-Object -ExpandProperty featureName
        $featureNames | Should -Contain $knownEnabledFeature
        $featureNames | Should -Contain $knownDisabledFeature
    }
}
