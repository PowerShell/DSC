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

        # Get the displayName for the known installed capability to use in wildcard displayName tests
        $fullInfoJson = '{"capabilities":[{"name":"' + $knownCapabilityNameOne + '","displayName":"*"}]}'
        $fullInfoOutput = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $fullInfoJson | ConvertFrom-Json
        $knownDisplayName = $fullInfoOutput.resources[0].properties.capabilities[0].displayName
        if (-not $knownDisplayName) {
            throw "Failed to get displayName for $knownCapabilityNameOne"
        }
        # Extract a substring from the displayName for wildcard matching (use first word if multi-word)
        $displayNameWords = $knownDisplayName -split '\s+'
        $knownDisplayNameWord = $displayNameWords[0]
    }

    It 'exports all capabilities with no input' -Skip:(!$isElevated) {
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        $capabilities.Count | Should -BeGreaterThan 0
        $capabilities[0].name | Should -Not -BeNullOrEmpty
        $capabilities[0].state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
    }

    It 'exports capabilities filtered by exact name' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"' + $knownCapabilityNameOne + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        $capabilities.Count | Should -Be 1
        $cap = $capabilities[0]
        $cap.name | Should -BeExactly $knownCapabilityNameOne
        $cap.state | Should -BeIn @(
            'NotPresent', 'UninstallPending', 'Staged', 'Removed',
            'Installed', 'InstallPending', 'Superseded', 'PartiallyInstalled'
        )
    }

    It 'exports capabilities filtered by wildcard name' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"Language.Basic*"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        foreach ($cap in $capabilities) {
            $cap.name | Should -BeLike 'Language.Basic*'
        }
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

    It 'exports capabilities with combined name and state filter' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"*","state":"Installed"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        foreach ($cap in $capabilities) {
            $cap.state | Should -BeExactly 'Installed'
        }
    }

    It 'exports capabilities filtered by wildcard displayName' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"displayName":"*' + $knownDisplayNameWord + '*"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        foreach ($cap in $capabilities) {
            $cap.displayName | Should -BeLike "*$knownDisplayNameWord*"
        }
    }

    It 'exports capabilities with multiple filters using OR logic' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"' + $knownCapabilityNameOne + '"},{"name":"' + $knownCapabilityNameTwo + '"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities | Should -Not -BeNullOrEmpty
        $names = $capabilities | ForEach-Object { $_.name }
        $names | Should -Contain $knownCapabilityNameOne
        $names | Should -Contain $knownCapabilityNameTwo
    }

    It 'returns empty results for non-matching wildcard filter' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"ZZZNonExistent*"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities.Count | Should -Be 0
    }

    It 'returns complete capability properties when full-info filter is used' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"' + $knownCapabilityNameOne + '","displayName":"*"}]}'
        $output = dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $capabilities = $output.resources[0].properties.capabilities
        $capabilities.Count | Should -Be 1
        $cap = $capabilities[0]
        $cap.name | Should -BeExactly $knownCapabilityNameOne
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
