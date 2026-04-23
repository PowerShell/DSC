# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/SecretStoreSecret - get operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $modulesAvailable = if ($IsWindows) {
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement') -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretStore')
        } else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/SecretStoreSecret'
        $testSecretName = 'DSC-SecretStore-Get-Test'

        if ($IsWindows -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement') -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretStore')) {
            # Ensure vault is configured for non-interactive use
            Set-SecretStoreConfiguration -Authentication None -Interaction None -Confirm:$false -Force -ErrorAction SilentlyContinue
            if (-not (Get-SecretVault -Name 'SecretStore' -ErrorAction SilentlyContinue)) {
                Register-SecretVault -Name 'SecretStore' -ModuleName 'Microsoft.PowerShell.SecretStore' -DefaultVault
            }
            # Create a known test secret
            Set-Secret -Name $testSecretName -Secret 'TestValue123' -ErrorAction SilentlyContinue
        }
    }

    AfterAll {
        if ($IsWindows -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement')) {
            Remove-Secret -Name $testSecretName -ErrorAction SilentlyContinue
        }
    }

    It 'returns an existing secret with metadata' -Skip:(!$modulesAvailable) {
        $input = @{ name = $testSecretName } | ConvertTo-Json -Compress
        $json = $input | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        $result.name | Should -BeExactly $testSecretName
        $result._exist | Should -BeTrue
        $result.PSObject.Properties.Name | Should -Contain 'secretType'
        $result.PSObject.Properties.Name | Should -Contain 'vaultName'
    }

    It 'returns _exist false for a non-existent secret' -Skip:(!$modulesAvailable) {
        $input = @{ name = 'DSC-NonExistent-Secret-XXXXXX' } | ConvertTo-Json -Compress
        $json = $input | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        $result.name | Should -BeExactly 'DSC-NonExistent-Secret-XXXXXX'
        $result._exist | Should -BeFalse
    }

    It 'fails when name is not provided' -Skip:(!$modulesAvailable) {
        $input = '{}' | dsc resource get -r $resourceType -f - 2>&1
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'returns secret type as a known enum value' -Skip:(!$modulesAvailable) {
        $input = @{ name = $testSecretName } | ConvertTo-Json -Compress
        $json = $input | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        if ($result._exist) {
            $result.secretType | Should -BeIn @('String', 'SecureString', 'ByteArray', 'PSCredential', 'Hashtable')
        }
    }

    It 'accepts optional vaultName parameter' -Skip:(!$modulesAvailable) {
        $input = @{ name = $testSecretName; vaultName = 'SecretStore' } | ConvertTo-Json -Compress
        $json = $input | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        $result.name | Should -BeExactly $testSecretName
    }
}
