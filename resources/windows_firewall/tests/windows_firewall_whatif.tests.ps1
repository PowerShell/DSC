# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'windows_firewall config whatif tests' -Skip:(!$isElevated -or !$hasNetSecurity) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
        $hasNetSecurity = $null -ne (Get-Command 'Get-NetFirewallRule' -ErrorAction SilentlyContinue)
    }

    BeforeAll {
        $testRuleName = 'DSC-WindowsFirewall-WhatIf-Test'

        function Initialize-TestFirewallRule {
            $existing = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
            if (-not $existing) {
                New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName -Direction Inbound -Action Allow -Protocol TCP -LocalPort 32456 | Out-Null
            }
        }

        function Get-RuleExists {
            param([string]$Name = $testRuleName)
            $null -ne (Get-NetFirewallRule -Name $Name -ErrorAction SilentlyContinue)
        }
    }

    AfterEach {
        Remove-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        Remove-NetFirewallRule -Name 'DSC-WindowsFirewall-WhatIf-Create-Test' -ErrorAction SilentlyContinue
    }

    It 'Can whatif create a new rule' -Skip:(!$isElevated -or !$hasNetSecurity) {
        $createRuleName = 'DSC-WindowsFirewall-WhatIf-Create-Test'
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

        $existsBefore = Get-RuleExists -Name $createRuleName

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.rules[0].name | Should -BeExactly $createRuleName
        $result.rules[0].direction | Should -BeExactly 'Inbound'
        $result.rules[0].action | Should -BeExactly 'Block'
        $result.rules[0]._metadata.whatIf | Should -Match "Would create firewall rule '$createRuleName'"

        # Assert no mutation happened
        $existsAfter = Get-RuleExists -Name $createRuleName
        $existsBefore | Should -Be $existsAfter
    }

    It 'Can whatif update an existing rule' -Skip:(!$isElevated -or !$hasNetSecurity) {
        Initialize-TestFirewallRule

        $json = @{
            rules = @(@{
                name        = $testRuleName
                description = 'WhatIf updated description'
                enabled     = $false
            })
        } | ConvertTo-Json -Compress -Depth 5

        $stateBefore = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.rules[0].name | Should -BeExactly $testRuleName
        $result.rules[0].description | Should -BeExactly 'WhatIf updated description'
        $result.rules[0].enabled | Should -BeFalse

        # Assert no mutation happened
        $stateAfter = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        $stateAfter.Description | Should -Be $stateBefore.Description
        $stateAfter.Enabled | Should -Be $stateBefore.Enabled
    }

    It 'Can whatif remove an existing rule using _exist is false' -Skip:(!$isElevated -or !$hasNetSecurity) {
        Initialize-TestFirewallRule

        $json = @{
            rules = @(@{
                name    = $testRuleName
                '_exist' = $false
            })
        } | ConvertTo-Json -Compress -Depth 5

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.rules[0].name | Should -BeExactly $testRuleName
        $result.rules[0]._exist | Should -BeFalse
        $result.rules[0]._metadata.whatIf | Should -Match "Would remove firewall rule '$testRuleName'"

        # Assert no mutation happened — rule should still exist
        Get-RuleExists | Should -BeTrue
    }

    It 'Can whatif multiple rules in a single request' -Skip:(!$isElevated -or !$hasNetSecurity) {
        Initialize-TestFirewallRule
        $createRuleName = 'DSC-WindowsFirewall-WhatIf-Create-Test'
        Remove-NetFirewallRule -Name $createRuleName -ErrorAction SilentlyContinue

        $json = @{
            rules = @(
                @{ name = $testRuleName; description = 'WhatIf multi test' }
                @{ name = $createRuleName; direction = 'Outbound'; action = 'Allow'; protocol = 17; enabled = $true }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.rules.Count | Should -Be 2

        # First rule — update projection, no create metadata
        $result.rules[0].name | Should -BeExactly $testRuleName
        $result.rules[0].description | Should -BeExactly 'WhatIf multi test'

        # Second rule — create projection, includes whatIf metadata
        $result.rules[1].name | Should -BeExactly $createRuleName
        $result.rules[1]._metadata.whatIf | Should -Match "Would create firewall rule '$createRuleName'"

        # Assert no mutations happened
        (Get-RuleExists) | Should -BeTrue
        (Get-RuleExists -Name $createRuleName) | Should -BeFalse
    }
}
