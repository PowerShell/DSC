# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/SecretStoreSecret - export operation' -Skip:(!$IsWindows) {
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
        $testSecretPrefix = 'DSC-SecretStore-Export-Test'
        $testSecretName1 = "${testSecretPrefix}-1"
        $testSecretName2 = "${testSecretPrefix}-2"

        if ($IsWindows -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement') -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretStore')) {
            # Ensure vault is configured for non-interactive use
            Set-SecretStoreConfiguration -Authentication None -Interaction None -Confirm:$false -Force -ErrorAction SilentlyContinue
            if (-not (Get-SecretVault -Name 'SecretStore' -ErrorAction SilentlyContinue)) {
                Register-SecretVault -Name 'SecretStore' -ModuleName 'Microsoft.PowerShell.SecretStore' -DefaultVault
            }
            # Create known test secrets
            Set-Secret -Name $testSecretName1 -Secret 'ExportValue1' -ErrorAction SilentlyContinue
            Set-Secret -Name $testSecretName2 -Secret 'ExportValue2' -ErrorAction SilentlyContinue
        }
    }

    AfterAll {
        if ($IsWindows -and
            (Get-Module -ListAvailable -Name 'Microsoft.PowerShell.SecretManagement')) {
            Remove-Secret -Name $testSecretName1 -ErrorAction SilentlyContinue
            Remove-Secret -Name $testSecretName2 -ErrorAction SilentlyContinue
        }
    }

    It 'exports all secrets' -Skip:(!$modulesAvailable) {
        $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $results = $raw | ConvertFrom-Json
        $results | Should -Not -BeNullOrEmpty
    }

    It 'exports secrets matching a name filter' -Skip:(!$modulesAvailable) {
        $input = @{ name = "${testSecretPrefix}*" } | ConvertTo-Json -Compress
        $raw = $input | dsc resource export -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $results = $raw | ConvertFrom-Json
        $names = @($results | ForEach-Object {
            if ($_.resources) { $_.resources[0].properties.name } else { $_.name }
        })
        $names | Should -Contain $testSecretName1
        $names | Should -Contain $testSecretName2
    }

    It 'returns empty when no secrets match filter' -Skip:(!$modulesAvailable) {
        $input = @{ name = 'DSC-NonExistent-Export-XXXXXX' } | ConvertTo-Json -Compress
        $raw = $input | dsc resource export -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        # Should either be empty or return no matching secrets
        if ($raw) {
            $results = @($raw | ConvertFrom-Json)
            $names = @($results | ForEach-Object {
                if ($_.resources) { $_.resources[0].properties.name } else { $_.name }
            })
            $names | Should -Not -Contain 'DSC-NonExistent-Export-XXXXXX'
        }
    }

    It 'exported secrets contain expected properties' -Skip:(!$modulesAvailable) {
        $input = @{ name = $testSecretName1 } | ConvertTo-Json -Compress
        $raw = $input | dsc resource export -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $results = @($raw | ConvertFrom-Json)
        $results.Count | Should -BeGreaterOrEqual 1
        $secret = if ($results[0].resources) { $results[0].resources[0].properties } else { $results[0] }
        $secret.name | Should -BeExactly $testSecretName1
        $secret._exist | Should -BeTrue
        $secret.PSObject.Properties.Name | Should -Contain 'secretType'
        $secret.PSObject.Properties.Name | Should -Contain 'vaultName'
    }
}
