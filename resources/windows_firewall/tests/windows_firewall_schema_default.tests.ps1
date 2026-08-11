# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - synthetic test with schema defaults' {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/FirewallRuleList'
        $testRuleName = 'DSC-WindowsFirewall-SchemaDefault-Test'

        # Ensure a known rule exists for testing
        $existing = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        if (-not $existing) {
            New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName `
                -Direction Inbound -Action Allow -Protocol TCP -LocalPort 32921 `
                -Enabled True -PolicyStore PersistentStore | Out-Null
        }
    }

    AfterAll {
        Remove-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
    }

    It 'unspecifiedRulesAction set to default "ignore" does not report as differing' -Skip:(!$isElevated) {
        $json = @{
            unspecifiedRulesAction = 'ignore'
            rules = @(@{
                name = $testRuleName
                direction = 'Inbound'
                action = 'Allow'
                protocol = 6
                localPorts = '32921'
                enabled = $true
            })
        } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $out | ConvertFrom-Json
        $result.inDesiredState | Should -Be $true
        $result.differingProperties | Should -Not -Contain 'unspecifiedRulesAction'
    }

    It 'unspecifiedRulesAction omitted does not report as differing' -Skip:(!$isElevated) {
        $json = @{
            rules = @(@{
                name = $testRuleName
                direction = 'Inbound'
                action = 'Allow'
                protocol = 6
                localPorts = '32921'
                enabled = $true
            })
        } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $out | ConvertFrom-Json
        $result.inDesiredState | Should -Be $true
        $result.differingProperties | Should -Not -Contain 'unspecifiedRulesAction'
    }

    It 'non-default unspecifiedRulesAction "disable" is reported as differing' -Skip:(!$isElevated) {
        $json = @{
            unspecifiedRulesAction = 'disable'
            rules = @(@{
                name = $testRuleName
                direction = 'Inbound'
                action = 'Allow'
                protocol = 6
                localPorts = '32921'
                enabled = $true
            })
        } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $out | ConvertFrom-Json
        $result.differingProperties | Should -Contain 'unspecifiedRulesAction'
    }

    It 'non-default unspecifiedRulesAction "remove" is reported as differing' -Skip:(!$isElevated) {
        $json = @{
            unspecifiedRulesAction = 'remove'
            rules = @(@{
                name = $testRuleName
                direction = 'Inbound'
                action = 'Allow'
                protocol = 6
                localPorts = '32921'
                enabled = $true
            })
        } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = $out | ConvertFrom-Json
        $result.differingProperties | Should -Contain 'unspecifiedRulesAction'
    }
}
