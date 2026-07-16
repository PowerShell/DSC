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

    It 'returns an error when export input is provided' -Skip:(!$isElevated) {
        $inputJson = '{"capabilities":[{"identity":"OpenSSH.Client~~~~0.0.1.0"}]}'
        dsc resource export -r Microsoft.Windows/FeatureOnDemandList -i $inputJson 2>$TESTDRIVE/error.log | Out-Null
        $LASTEXITCODE | Should -Be 2
        (Get-Content -Raw $TESTDRIVE/error.log) | Should -Match 'does not support export filtering'
    }
}
