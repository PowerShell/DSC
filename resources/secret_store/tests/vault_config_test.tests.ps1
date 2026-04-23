# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/SecretStoreVaultConfig - test operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $modulesAvailable = if ($IsWindows) {
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement') -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretStore')
        } else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/SecretStoreVaultConfig'

        # Get current configuration to use as baseline for test assertions
        if ($IsWindows -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement') -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretStore')) {
            $getJson = '{}' | dsc resource get -r $resourceType -f - 2>$null
            $currentConfig = ($getJson | ConvertFrom-Json).actualState
        }
    }

    It 'returns true when configuration matches desired state' -Skip:(!$modulesAvailable) {
        # Test with current actual values so it should match
        $input = @{} | ConvertTo-Json -Compress
        if ($currentConfig.authentication) {
            $input = @{ authentication = $currentConfig.authentication } | ConvertTo-Json -Compress
        }
        $json = $input | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $json | ConvertFrom-Json
        $result.inDesiredState | Should -BeTrue
    }

    It 'returns false when authentication does not match' -Skip:(!$modulesAvailable) {
        # Use the opposite of the current authentication value
        $desiredAuth = if ($currentConfig.authentication -eq 'Password') { 'None' } else { 'Password' }
        $input = @{ authentication = $desiredAuth } | ConvertTo-Json -Compress
        $json = $input | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $json | ConvertFrom-Json
        $result.inDesiredState | Should -BeFalse
    }

    It 'returns false when password timeout does not match' -Skip:(!$modulesAvailable) {
        # Use a different timeout than current
        $desiredTimeout = if ($currentConfig.passwordTimeout -eq 999) { 1000 } else { 999 }
        $input = @{ passwordTimeout = $desiredTimeout } | ConvertTo-Json -Compress
        $json = $input | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $json | ConvertFrom-Json
        $result.inDesiredState | Should -BeFalse
    }

    It 'returns true when no properties are specified' -Skip:(!$modulesAvailable) {
        $input = '{}' | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $input | ConvertFrom-Json
        $result.inDesiredState | Should -BeTrue
    }
}
