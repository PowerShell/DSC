# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FeatureOnDemandList - get operation' -Skip:(!$IsWindows) {
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

    It 'gets a known capability by identity' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '"}]}'
        $output = dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.capabilities | Should -Not -BeNullOrEmpty
        $output.actualState.capabilities.Count | Should -Be 1
        $cap = $output.actualState.capabilities[0]
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

    It 'gets multiple capabilities in a single request' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '"},{"identity":"' + $knownCapabilityNameTwo + '"}]}'
        $output = dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.capabilities.Count | Should -Be 2
        $output.actualState.capabilities[0].identity | Should -BeExactly $knownCapabilityNameOne
        $output.actualState.capabilities[1].identity | Should -BeExactly $knownCapabilityNameTwo
    }

    It 'returns error when identity is missing' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"state":"Installed"}]}'
        $testError = & { dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'identity is required'
    }

    It 'returns error when capabilities array is empty' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[]}'
        $testError = & { dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'cannot be empty'
    }

    It 'returns error for malformed JSON input' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{invalid'
        $testError = & { dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns _exist false for a non-existent capability identity' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"NonExistent-Capability-1234567890"}]}'
        $output = dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.capabilities | Should -Not -BeNullOrEmpty
        $output.actualState.capabilities.Count | Should -Be 1
        $cap = $output.actualState.capabilities[0]
        $cap.identity | Should -BeExactly 'NonExistent-Capability-1234567890'
        $cap._exist | Should -BeFalse
        $cap.state | Should -BeNullOrEmpty
        $cap.displayName | Should -BeNullOrEmpty
        $cap.description | Should -BeNullOrEmpty
    }

    It 'returns _exist false alongside valid capabilities' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapabilityNameOne + '"},{"identity":"NonExistent-Capability-1234567890"}]}'
        $output = dsc resource get -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.actualState.capabilities.Count | Should -Be 2
        $output.actualState.capabilities[0].identity | Should -BeExactly $knownCapabilityNameOne
        $output.actualState.capabilities[0].PSObject.Properties.Name | Should -Not -Contain '_exist'
        $output.actualState.capabilities[1].identity | Should -BeExactly 'NonExistent-Capability-1234567890'
        $output.actualState.capabilities[1]._exist | Should -BeFalse
    }
}
