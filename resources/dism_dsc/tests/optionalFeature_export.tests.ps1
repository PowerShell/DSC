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

    It 'exports features filtered by exact featureName' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"' + $knownFeatureNameOne + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        $features.Count | Should -Be 1
        $feature = $features[0]
        $feature.featureName | Should -BeExactly $knownFeatureNameOne
        $feature.displayName | Should -Not -BeNullOrEmpty
        $feature.description | Should -Not -BeNullOrEmpty
        $feature.restartRequired | Should -BeIn @('No', 'Possible', 'Required')
    }

    It 'exports features filtered by wildcard featureName' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"Printing-*"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        foreach ($feature in $features) {
            $feature.featureName | Should -BeLike 'Printing-*'
        }
    }

    It 'exports features filtered by state' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"state":"Installed"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        foreach ($feature in $features) {
            $feature.state | Should -BeExactly 'Installed'
        }
    }

    It 'exports features with combined featureName and state filter' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"*","state":"Installed"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        foreach ($feature in $features) {
            $feature.state | Should -BeExactly 'Installed'
        }
    }

    It 'exports features filtered by wildcard displayName' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"displayName":"*Print*"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        foreach ($feature in $features) {
            $feature.displayName | Should -BeLike '*Print*'
        }
    }

    It 'exports features with multiple filters using OR logic' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"' + $knownFeatureNameOne + '"},{"featureName":"' + $knownFeatureNameTwo + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features | Should -Not -BeNullOrEmpty
        $names = $features | ForEach-Object { $_.featureName }
        $names | Should -Contain $knownFeatureNameOne
        $names | Should -Contain $knownFeatureNameTwo
    }

    It 'returns empty results for non-matching wildcard filter' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"ZZZNonExistent*"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features.Count | Should -Be 0
    }

    It 'returns complete feature properties in export results' -Skip:(!$isElevated) {
        $inputJson = '{"features":[{"featureName":"' + $knownFeatureNameOne + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/OptionalFeatureList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $features = $output.resources[0].properties.features
        $features.Count | Should -Be 1
        $feature = $features[0]
        $feature.featureName | Should -BeExactly $knownFeatureNameOne
        $feature.state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
        $feature.displayName | Should -Not -BeNullOrEmpty
        $feature.description | Should -Not -BeNullOrEmpty
        $feature.restartRequired | Should -BeIn @('No', 'Possible', 'Required')
    }
}
