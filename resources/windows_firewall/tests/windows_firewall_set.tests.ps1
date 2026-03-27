# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - set operation' -Skip:(!$IsWindows) {
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
        $testRuleName = 'DSC-WindowsFirewall-Set-Test'

        function Initialize-TestFirewallRule {
            $existing = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
            if (-not $existing) {
                New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName -Direction Inbound -Action Allow -Protocol TCP -LocalPort 32123 | Out-Null
            }
        }

        function Get-RuleState {
            param(
                [string]$Name = $testRuleName
            )
            $json = @{ rules = @(@{ name = $Name }) } | ConvertTo-Json -Compress -Depth 5
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState.rules[0]
        }

        Initialize-TestFirewallRule
    }

    AfterAll {
        Remove-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        Remove-NetFirewallRule -Name 'DSC-WindowsFirewall-Create-Test' -ErrorAction SilentlyContinue
    }

    It 'fails when name is not provided' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ enabled = $true }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource set -r $resourceType -f - 2>&1
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'fails when rules array is empty' -Skip:(!$isElevated) {
        $json = '{"rules":[]}'
        $out = $json | dsc resource set -r $resourceType -f - 2>&1
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'updates an existing rule' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule
        $json = @{ rules = @(@{ name = $testRuleName; description = 'Updated by DSC test'; enabled = $false }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).afterState.rules[0]
        $result.name | Should -BeExactly $testRuleName
        $result.description | Should -BeExactly 'Updated by DSC test'
        $result.enabled | Should -BeFalse
    }

    It 'creates a new rule when it does not exist' -Skip:(!$isElevated) {
        $createRuleName = 'DSC-WindowsFirewall-Create-Test'
        Remove-NetFirewallRule -Name $createRuleName -ErrorAction SilentlyContinue

        $json = @{
            rules = @(@{
                name      = $createRuleName
                direction = 'Inbound'
                action    = 'Block'
                protocol  = 6
                enabled   = $true
            })
        } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).afterState.rules[0]
        $result.name | Should -BeExactly $createRuleName
        $result.direction | Should -BeExactly 'Inbound'
        $result.action | Should -BeExactly 'Block'
        $result.enabled | Should -BeTrue

        Remove-NetFirewallRule -Name $createRuleName -ErrorAction SilentlyContinue
    }

    It 'clears ports when switching protocol from TCP to ICMP' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule
        # The test rule is TCP with LocalPort 32123. Switch to ICMPv4 (protocol 1).
        $json = @{ rules = @(@{ name = $testRuleName; protocol = 1 }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).afterState.rules[0]
        $result.protocol | Should -Be 1
        $result.PSObject.Properties.Name | Should -Not -Contain 'localPorts'

        # Restore original state for subsequent tests
        $json = @{ rules = @(@{ name = $testRuleName; protocol = 6; localPorts = '32123' }) } | ConvertTo-Json -Compress -Depth 5
        $json | dsc resource set -r $resourceType -f - 2>$null | Out-Null
    }

    It 'removes an existing rule when _exist is false' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule
        $json = @{ rules = @(@{ name = $testRuleName; _exist = $false }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).afterState.rules[0]
        $result.name | Should -BeExactly $testRuleName
        $result._exist | Should -BeFalse

        $actualRule = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        $actualRule | Should -BeNullOrEmpty

        Initialize-TestFirewallRule
        $state = Get-RuleState
        $state.PSObject.Properties.Name | Should -Not -Contain '_exist'
    }

    It 'handles multiple rules in a single request' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule
        $secondRuleName = 'DSC-WindowsFirewall-Create-Test'
        Remove-NetFirewallRule -Name $secondRuleName -ErrorAction SilentlyContinue

        $json = @{
            rules = @(
                @{ name = $testRuleName; description = 'Multi rule test' }
                @{ name = $secondRuleName; direction = 'Outbound'; action = 'Allow'; protocol = 6; enabled = $true }
            )
        } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $results = ($out | ConvertFrom-Json).afterState.rules
        $results.Count | Should -Be 2
        $results[0].description | Should -BeExactly 'Multi rule test'
        $results[1].name | Should -BeExactly $secondRuleName

        Remove-NetFirewallRule -Name $secondRuleName -ErrorAction SilentlyContinue
    }
}
