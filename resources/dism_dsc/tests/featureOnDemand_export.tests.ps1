# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FeatureOnDemandList - export operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    BeforeAll {
        # Use dism command to get known capability names
        $dismOutput = & dism /Online /Get-Capabilities /Format:Table /English 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to get capabilities using dism: $dismOutput"
        }
        $installedMatches = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Installed\s*$'
        $notPresentMatches = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Not Present\s*$'
        if (-not $installedMatches -or -not $notPresentMatches) {
            throw "Failed to find both installed and not-present capabilities in DISM output.`nOutput:`n$dismOutput"
        }
        $knownCapabilityNameOne = $installedMatches[0].Matches[0].Groups[1].Value
        $knownCapabilityNameTwo = $notPresentMatches[0].Matches[0].Groups[1].Value

    }

    It 'exports all capabilities with no input' -Skip:(!$isElevated) {
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        $capabilities.Count | Should -BeGreaterThan 0
        $capabilities[0].identity | Should -Not -BeNullOrEmpty
        $capabilities[0].state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
    }

    It 'exports capabilities filtered by exact identity' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        $capabilities.Count | Should -Be 1
        $cap = $capabilities[0]
        $cap.identity | Should -BeExactly $knownCapabilityNameOne
        $cap.state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
    }

    It 'treats wildcard characters as literal identity characters' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '*"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.resources[0].properties.capabilities.Count | Should -Be 0
    }

    It 'exports capabilities filtered by state' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"state":"Installed"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        foreach ($cap in $capabilities) {
            $cap.state | Should -BeExactly 'Installed'
        }
    }

    It 'exports capabilities with multiple filters using OR logic' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '"},{"identity":"' + $knownCapabilityNameTwo + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        $identities = $capabilities | ForEach-Object { $_.identity }
        $identities | Should -Contain $knownCapabilityNameOne
        $identities | Should -Contain $knownCapabilityNameTwo
    }

    It 'returns empty results for a non-matching identity filter' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"ZZZNonExistent"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities.Count | Should -Be 0
    }

    It 'returns empty results for empty capabilities array input' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities.Count | Should -Be 0
    }

    It 'returns complete capability properties when an exact displayName filter is used' -Skip:(!$isElevated) {
        $getInputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '"}]}'
        $knownDisplayName = (dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $getInputJson | ConvertFrom-Json).actualState.capabilities[0].displayName
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '","displayName":"' + $knownDisplayName + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities.Count | Should -Be 1
        $cap = $capabilities[0]
        $cap.identity | Should -BeExactly $knownCapabilityNameOne
        $cap.state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
        $cap.displayName | Should -Not -BeNullOrEmpty
        $cap.description | Should -Not -BeNullOrEmpty
        $cap.downloadSize | Should -Not -BeNullOrEmpty
        $cap.installSize | Should -Not -BeNullOrEmpty
    }
}
