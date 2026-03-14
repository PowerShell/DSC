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
        # Dynamically discover capability names instead of hardcoding
        $dismOutput = & dism /Online /Get-Capabilities /Format:Table /English 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to get capabilities using dism: $dismOutput"
        }
        $installedMatches = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Installed\s*$'
        $notPresentMatches = $dismOutput | Select-String -Pattern '^\s*(\S+)\s+\|\s+Not Present\s*$'
        if (-not $installedMatches -or -not $notPresentMatches) {
            throw "Failed to find both installed and not-present capabilities in DISM output.`nOutput:`n$dismOutput"
        }
        $knownInstalledCapability = $installedMatches[0].Matches[0].Groups[1].Value
        $knownNotPresentCapability = $notPresentMatches[0].Matches[0].Groups[1].Value

        # Record initial states for cleanup
        $script:initialInstalledState = 'Installed'
        $script:initialNotPresentState = 'NotPresent'
    }

    AfterAll {
        # Restore capabilities to their original states
        if ($knownInstalledCapability) {
            $restoreJson = '{"capabilities":[{"name":"' + $knownInstalledCapability + '","state":"' + $script:initialInstalledState + '"}]}'
            dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $restoreJson 2>&1 | Out-Null
        }
        if ($knownNotPresentCapability) {
            $restoreJson = '{"capabilities":[{"name":"' + $knownNotPresentCapability + '","state":"' + $script:initialNotPresentState + '"}]}'
            dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $restoreJson 2>&1 | Out-Null
        }
    }

    It 'returns error when name is missing' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"state":"Installed"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'name is required'
    }

    It 'returns error when state is missing' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"' + $knownInstalledCapability + '"}]}'
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
        $inputJson = '{"capabilities":[{"name":"' + $knownInstalledCapability + '","state":"Staged"}]}'
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
        $inputJson = '{"capabilities":[{"name":"NonExistent-Capability-1234567890","state":"Installed"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
        "$testError" | Should -Match 'not found'
    }

    It 'can install a capability and returns updated state' -Skip:(!$isElevated) -Tag 'Mutating' {
        $inputJson = '{"capabilities":[{"name":"' + $knownInstalledCapability + '","state":"Installed"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $output.afterState.capabilities.Count | Should -Be 1
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly $knownInstalledCapability
        $cap.state | Should -BeIn @('Installed', 'InstallPending')
        $cap.displayName | Should -Not -BeNullOrEmpty
    }

    It 'can remove a capability and returns updated state' -Skip:(!$isElevated) -Tag 'Mutating' {
        $inputJson = '{"capabilities":[{"name":"' + $knownNotPresentCapability + '","state":"NotPresent"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $output.afterState.capabilities.Count | Should -Be 1
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly $knownNotPresentCapability
        $cap.state | Should -BeIn @('NotPresent', 'Removed', 'UninstallPending', 'Staged')
    }

    It 'sets state to Installed for an already installed capability' -Skip:(!$isElevated) -Tag 'Mutating' {
        # Set Installed on an already-installed capability — should succeed idempotently
        $installJson = '{"capabilities":[{"name":"' + $knownInstalledCapability + '","state":"Installed"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $installJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly $knownInstalledCapability
        $cap.state | Should -Be 'Installed'
    }

    It 'sets state to NotPresent for an already not-present capability' -Skip:(!$isElevated) -Tag 'Mutating' {
        # Set NotPresent on an already not-present capability — should succeed idempotently
        $notPresentJson = '{"capabilities":[{"name":"' + $knownNotPresentCapability + '","state":"NotPresent"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $notPresentJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly $knownNotPresentCapability
        $cap.state | Should -BeIn @('NotPresent', 'Removed', 'Staged')

        # Set NotPresent again — should still succeed idempotently
        $output2 = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $notPresentJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $cap2 = $output2.afterState.capabilities[0]
        $cap2.state | Should -BeIn @('NotPresent', 'Removed', 'Staged')
    }
}
