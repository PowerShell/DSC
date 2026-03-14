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

    It 'returns error when name is missing' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"state":"Installed"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error when state is missing' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"Language.Basic~~~en-US~0.0.1.0"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error when capabilities array is empty' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns error for unsupported desired state' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"Language.Basic~~~en-US~0.0.1.0","state":"Staged"}]}'
        $testError = & { dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>&1 }
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'can install a capability and returns updated state' -Skip:(!$isElevated) {
        # Use a capability known to exist on most Windows systems
        $inputJson = '{"capabilities":[{"name":"Language.Basic~~~en-US~0.0.1.0","state":"Installed"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $output.afterState.capabilities.Count | Should -Be 1
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly 'Language.Basic~~~en-US~0.0.1.0'
        $cap.state | Should -BeIn @('Installed', 'InstallPending')
        $cap.displayName | Should -Not -BeNullOrEmpty
    }

    It 'can remove a capability and returns updated state' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"name":"Language.Basic~~~en-US~0.0.1.0","state":"NotPresent"}]}'
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $inputJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $output.afterState.capabilities.Count | Should -Be 1
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly 'Language.Basic~~~en-US~0.0.1.0'
        $cap.state | Should -BeIn @('NotPresent', 'Removed', 'UninstallPending', 'Staged')
    }

    It 'sets state to Installed for an already installed capability' -Skip:(!$isElevated) {
        # First ensure the capability is installed
        $installJson = '{"capabilities":[{"name":"Language.Basic~~~en-US~0.0.1.0","state":"Installed"}]}'
        dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $installJson | Out-Null
        $LASTEXITCODE | Should -Be 0

        # Set Installed again — should succeed idempotently
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $installJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly 'Language.Basic~~~en-US~0.0.1.0'
        $cap.state | Should -Be 'Installed'
    }

    It 'sets state to NotPresent for an already not-present capability' -Skip:(!$isElevated) {
        # First ensure the capability is not present
        $removeJson = '{"capabilities":[{"name":"Language.Basic~~~en-US~0.0.1.0","state":"NotPresent"}]}'
        dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $removeJson | Out-Null
        $LASTEXITCODE | Should -Be 0

        # Set NotPresent again — should succeed idempotently
        $output = dsc resource set -r Microsoft.Windows/FeatureOnDemandList -i $removeJson | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $output.afterState.capabilities | Should -Not -BeNullOrEmpty
        $cap = $output.afterState.capabilities[0]
        $cap.name | Should -BeExactly 'Language.Basic~~~en-US~0.0.1.0'
        $cap.state | Should -BeIn @('NotPresent', 'Removed', 'Staged')
    }
}
