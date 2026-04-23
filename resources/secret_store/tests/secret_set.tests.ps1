# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/SecretStoreSecret - set operation' -Skip:(!$IsWindows) {
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
        $testSecretName = 'DSC-SecretStore-Set-Test'
        $testSecretName2 = 'DSC-SecretStore-Set-Test-2'

        if ($IsWindows -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement') -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretStore')) {
            # Ensure vault is configured for non-interactive use
            Set-SecretStoreConfiguration -Authentication None -Interaction None -Confirm:$false -Force -ErrorAction SilentlyContinue
            if (-not (Get-SecretVault -Name 'SecretStore' -ErrorAction SilentlyContinue)) {
                Register-SecretVault -Name 'SecretStore' -ModuleName 'Microsoft.PowerShell.SecretStore' -DefaultVault
            }
        }
    }

    AfterAll {
        if ($IsWindows -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement')) {
            Remove-Secret -Name $testSecretName -ErrorAction SilentlyContinue
            Remove-Secret -Name $testSecretName2 -ErrorAction SilentlyContinue
        }
    }

    It 'creates a new secret' -Skip:(!$modulesAvailable) {
        Remove-Secret -Name $testSecretName -ErrorAction SilentlyContinue

        $input = @{ name = $testSecretName; value = 'NewSecretValue' } | ConvertTo-Json -Compress
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.name | Should -BeExactly $testSecretName
        $result._exist | Should -BeTrue
    }

    It 'updates an existing secret' -Skip:(!$modulesAvailable) {
        # Ensure secret exists first
        Set-Secret -Name $testSecretName -Secret 'OldValue' -ErrorAction SilentlyContinue

        $input = @{ name = $testSecretName; value = 'UpdatedValue' } | ConvertTo-Json -Compress
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.name | Should -BeExactly $testSecretName
        $result._exist | Should -BeTrue
    }

    It 'removes a secret when _exist is false' -Skip:(!$modulesAvailable) {
        # Ensure secret exists first
        Set-Secret -Name $testSecretName2 -Secret 'ToBeRemoved' -ErrorAction SilentlyContinue

        $input = @{ name = $testSecretName2; _exist = $false } | ConvertTo-Json -Compress
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.name | Should -BeExactly $testSecretName2
        $result._exist | Should -BeFalse

        # Verify the secret is actually gone
        $info = Get-SecretInfo -Name $testSecretName2 -ErrorAction SilentlyContinue
        $info | Should -BeNullOrEmpty
    }

    It 'sets metadata on a secret' -Skip:(!$modulesAvailable) {
        Remove-Secret -Name $testSecretName -ErrorAction SilentlyContinue

        $input = @{
            name     = $testSecretName
            value    = 'MetadataTest'
            metadata = @{ environment = 'test'; owner = 'dsc' }
        } | ConvertTo-Json -Compress -Depth 5
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.name | Should -BeExactly $testSecretName
        $result._exist | Should -BeTrue
        $result.metadata.environment | Should -BeExactly 'test'
        $result.metadata.owner | Should -BeExactly 'dsc'
    }

    It 'fails when name is not provided' -Skip:(!$modulesAvailable) {
        $input = @{ value = 'NoName' } | ConvertTo-Json -Compress
        $input | dsc resource set -r $resourceType -f - 2>&1
        $LASTEXITCODE | Should -Not -Be 0
    }
}
