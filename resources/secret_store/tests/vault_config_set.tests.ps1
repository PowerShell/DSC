# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/SecretStoreVaultConfig - set operation' -Skip:(!$IsWindows) {
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
    }

    It 'sets authentication type' -Skip:(!$modulesAvailable) {
        $input = @{ authentication = 'None'; interaction = 'None' } | ConvertTo-Json -Compress
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.authentication | Should -BeExactly 'None'
    }

    It 'sets password timeout' -Skip:(!$modulesAvailable) {
        $input = @{ passwordTimeout = 600 } | ConvertTo-Json -Compress
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.passwordTimeout | Should -Be 600
    }

    It 'sets interaction preference' -Skip:(!$modulesAvailable) {
        $input = @{ interaction = 'None' } | ConvertTo-Json -Compress
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.interaction | Should -BeExactly 'None'
    }

    It 'returns modules installed and vault registered after set' -Skip:(!$modulesAvailable) {
        $input = @{ authentication = 'None'; interaction = 'None' } | ConvertTo-Json -Compress
        $json = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).afterState
        $result.secretManagementInstalled | Should -BeTrue
        $result.secretStoreInstalled | Should -BeTrue
        $result.vaultRegistered | Should -BeTrue
    }

    It 'is idempotent when setting the same configuration twice' -Skip:(!$modulesAvailable) {
        $input = @{ authentication = 'None'; interaction = 'None'; passwordTimeout = 900 } | ConvertTo-Json -Compress

        $json1 = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $json2 = $input | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result1 = ($json1 | ConvertFrom-Json).afterState
        $result2 = ($json2 | ConvertFrom-Json).afterState
        $result1.authentication | Should -BeExactly $result2.authentication
        $result1.passwordTimeout | Should -Be $result2.passwordTimeout
        $result1.interaction | Should -BeExactly $result2.interaction
    }
}
