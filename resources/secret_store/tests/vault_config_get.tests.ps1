# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/SecretStoreVaultConfig - get operation' -Skip:(!$IsWindows) {
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

    It 'returns module installation status' {
        $json = '{}' | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        $result.PSObject.Properties.Name | Should -Contain 'secretManagementInstalled'
        $result.PSObject.Properties.Name | Should -Contain 'secretStoreInstalled'
        $result.secretManagementInstalled | Should -BeOfType [bool]
        $result.secretStoreInstalled | Should -BeOfType [bool]
    }

    It 'returns _exist false when modules are not installed' -Skip:($modulesAvailable) {
        $json = '{}' | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        $result._exist | Should -BeFalse
    }

    It 'returns vault registration and configuration when modules are installed' -Skip:(!$modulesAvailable) {
        $json = '{}' | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        $result.PSObject.Properties.Name | Should -Contain 'vaultRegistered'
        $result.vaultRegistered | Should -BeOfType [bool]
    }

    It 'returns authentication and interaction when vault is configured' -Skip:(!$modulesAvailable) {
        $json = '{}' | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        if ($result._exist -ne $false) {
            $result.authentication | Should -BeIn @('None', 'Password')
            $result.interaction | Should -BeIn @('Prompt', 'None')
            $result.passwordTimeout | Should -BeOfType [int]
        }
    }

    It 'accepts input json without error' {
        $input = @{ authentication = 'Password' } | ConvertTo-Json -Compress
        $json = $input | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($json | ConvertFrom-Json).actualState
        $result | Should -Not -BeNullOrEmpty
    }
}
