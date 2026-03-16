# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FeatureOnDemandList - set operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    BeforeAll {
        function Get-CapabilityState {
            param($capabilityName)
            $dismOutput = dism /Online /Get-CapabilityInfo /CapabilityName:$capabilityName 2>&1 | Out-String
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to get capability info for $capabilityName : $($dismOutput)"
            }
            if ($dismOutput -match 'State : (?<state>.+)') {
                return $matches['state'].Trim()
            } else {
                throw "Failed to parse capability state for $capabilityName. DISM output: $($dismOutput)"
            }
        }

        # Use SNMP as known capability since it is small
        $knownCapability = 'SNMP.Client~~~~0.0.1.0'

        # Get current state from dism.exe
        $script:initialInstalledState = Get-CapabilityState -capabilityName $knownCapability
        if (-not $script:initialInstalledState) {
            throw "Failed to parse capability state for $knownCapability during test setup. DISM output: $($dismOutput)"
        }
    }

    AfterAll {
        # restore original state to ensure we don't leave the system in a modified state after tests
        if ($script:initialInstalledState -eq 'Installed' -and (Get-CapabilityState -capabilityName $knownCapability) -ne 'Installed') {
            $output = dism /Online /Add-Capability /CapabilityName:$knownCapability /NoRestart 2>&1
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to restore capability $knownCapability to Installed state after tests: $($output)"
            }
        } elseif ($script:initialInstalledState -ne 'Installed' -and (Get-CapabilityState -capabilityName $knownCapability) -eq 'Installed') {
            $output = dism /Online /Remove-Capability /CapabilityName:$knownCapability /NoRestart 2>&1
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to restore capability $knownCapability to NotPresent state after tests: $($output)"
            }
        }
    }

    It 'returns error when identity is missing' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"state":"Installed"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'identity is required'
    }

    It 'returns error when state is missing' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapability + '"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'state is required'
    }

    It 'returns error when capabilities array is empty' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'cannot be empty'
    }

    It 'returns error for unsupported desired state' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"' + $knownCapability + '","state":"Staged"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'Unsupported desired state'
    }

    It 'returns error for malformed JSON input' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{invalid'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error when installing a non-existent capability' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"NonExistent-Capability-1234567890","state":"Installed"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'not found'
    }

    It 'can install a capability and returns updated state' -Skip:(!$isElevated) -Tag 'Mutating' {
        if ($script:initialInstalledState -eq 'Installed') {
            # uninstall the capability first to ensure we can test installing it
            $output = dism /Online /Remove-Capability /CapabilityName:$knownCapability /NoRestart 2>&1
            $LASTEXITCODE | Should -Be 0 -Because "Failed to disable capability $knownCapability for setup: $($output)"
        }

        $inputJson = '{"capabilities":[{"identity":"' + $knownCapability + '","state":"Installed"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $output.afterState.capabilities.Count | Should -Be 1
        $cap = $output.afterState.capabilities[0]
        $cap.identity | Should -BeExactly $knownCapability
        $cap.state | Should -BeIn @('Installed', 'InstallPending')
        $cap.displayName | Should -Not -BeNullOrEmpty
    }

    It 'sets state to Installed for an already installed capability' -Skip:(!$isElevated) -Tag 'Mutating' {
        if ($script:initialInstalledState -ne 'Installed') {
            # install the capability first to ensure we can test setting it to Installed again
            $output = dism /Online /Add-Capability /CapabilityName:$knownCapability /NoRestart 2>&1
            $LASTEXITCODE | Should -Be 0 -Because "Failed to enable capability $knownCapability for setup: $($output)"
        }

        $installJson = '{"capabilities":[{"identity":"' + $knownCapability + '","state":"Installed"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $installJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because "Failed to set already installed capability $knownCapability to Installed: $($output | ConvertTo-Json -Depth 5)"
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $cap = $output.afterState.capabilities[0]
        $cap.identity | Should -BeExactly $knownCapability
        $cap.state | Should -BeIn @('Installed', 'InstallPending')
    }

    It 'can remove a capability and returns updated state' -Skip:(!$isElevated) -Tag 'Mutating' {
        if ($script:initialInstalledState -ne 'Installed') {
            # install the capability first to ensure we can test removing it
            $output = dism /Online /Add-Capability /CapabilityName:$knownCapability /NoRestart 2>&1
            $LASTEXITCODE | Should -Be 0 -Because "Failed to enable capability $knownCapability for setup: $($output)"
        }

        $inputJson = '{"capabilities":[{"identity":"' + $knownCapability + '","state":"NotPresent"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $output.afterState.capabilities.Count | Should -Be 1
        $cap = $output.afterState.capabilities[0]
        $cap.identity | Should -BeExactly $knownCapability
        $cap.state | Should -BeIn @('NotPresent', 'Removed', 'UninstallPending', 'Staged')
    }

    It 'sets state to NotPresent for an already not-present capability' -Skip:(!$isElevated) -Tag 'Mutating' {
        if ($script:initialInstalledState -eq 'Installed') {
            # uninstall the capability first to ensure we can test setting it to NotPresent again
            $output = dism /Online /Remove-Capability /CapabilityName:$knownCapability /NoRestart 2>&1
            $LASTEXITCODE | Should -Be 0 -Because "Failed to disable capability $knownCapability for setup: $($output)"
        }

        $notPresentJson = '{"capabilities":[{"identity":"' + $knownCapability + '","state":"NotPresent"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $notPresentJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because "Failed to set already not-present capability $knownCapability to NotPresent: $($output | ConvertTo-Json -Depth 5)"
        $cap = $output.afterState.capabilities[0]
        $cap.state | Should -BeIn @('NotPresent', 'UninstallPending')
    }
}
