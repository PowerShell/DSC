# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - set operation' -Skip:(!$isElevated) {
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

Describe 'Microsoft.Windows/FirewallRuleList - unspecifiedRulesAction (what-if)' -Skip:(!$isElevated) {
    BeforeDiscovery {
        $isElevated = if ($IsWindows) {
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
        } else {
            $false
        }
    }

    BeforeAll {
        $testRuleName = 'DSC-WindowsFirewall-Unspecified-Test'

        function Initialize-TestFirewallRule {
            $existing = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
            if (-not $existing) {
                New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName -Direction Inbound -Action Allow -Protocol TCP -LocalPort 32789 -Enabled True | Out-Null
            } else {
                Set-NetFirewallRule -Name $testRuleName -Enabled True
            }
        }

        Initialize-TestFirewallRule
    }

    AfterAll {
        Remove-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
    }

    It 'does not affect unspecified rules when unspecifiedRulesAction is ignore' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule

        # Specify a different rule name so $testRuleName is "unspecified"
        $json = @{
            unspecifiedRulesAction = 'ignore'
            rules = @(@{
                name      = 'SomeOtherRuleThatMayNotExist'
                direction = 'Inbound'
                action    = 'Allow'
                protocol  = 6
                enabled   = $true
            })
        } | ConvertTo-Json -Compress -Depth 5

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        # Only the specified rule should appear in results; no unspecified rules affected
        $unspecifiedEntries = $result.rules | Where-Object { $_.name -eq $testRuleName }
        $unspecifiedEntries | Should -BeNullOrEmpty
    }

    It 'does not affect unspecified rules when unspecifiedRulesAction is omitted' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule

        $json = @{
            rules = @(@{
                name      = 'SomeOtherRuleThatMayNotExist'
                direction = 'Inbound'
                action    = 'Allow'
                protocol  = 6
                enabled   = $true
            })
        } | ConvertTo-Json -Compress -Depth 5

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        # No unspecified rules in results
        $unspecifiedEntries = $result.rules | Where-Object { $_.name -eq $testRuleName }
        $unspecifiedEntries | Should -BeNullOrEmpty
    }

    It 'reports would disable unspecified rules when unspecifiedRulesAction is disable' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule

        # Specify only testRuleName; all other rules are "unspecified" and should be disabled
        $json = @{
            unspecifiedRulesAction = 'disable'
            rules = @(@{
                name    = $testRuleName
                enabled = $true
            })
        } | ConvertTo-Json -Compress -Depth 5

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        # The specified rule should appear without disable metadata
        $specifiedEntry = $result.rules | Where-Object { $_.name -eq $testRuleName -and $null -eq $_._metadata }
        $specifiedEntry | Should -Not -BeNullOrEmpty

        # At least one unspecified rule should have disable what-if metadata
        $disabledEntries = $result.rules | Where-Object { $_._metadata.whatIf -match 'Would disable unspecified firewall rule' }
        $disabledEntries.Count | Should -BeGreaterThan 0

        # All disabled entries should have enabled = false
        foreach ($entry in $disabledEntries) {
            $entry.enabled | Should -BeFalse
        }

        # Verify no actual changes: the test rule is still enabled
        $actual = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        $actual.Enabled | Should -Be 'True'
    }

    It 'skips already-disabled rules when unspecifiedRulesAction is disable' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule
        # Disable the test rule so it is already disabled
        Set-NetFirewallRule -Name $testRuleName -Enabled False

        # Use a rule name that matches nothing, making testRuleName "unspecified"
        $otherRuleName = 'DSC-WindowsFirewall-Unspecified-Other'
        New-NetFirewallRule -Name $otherRuleName -DisplayName $otherRuleName -Direction Inbound -Action Allow -Protocol TCP -LocalPort 32790 -Enabled True -ErrorAction SilentlyContinue | Out-Null

        $json = @{
            unspecifiedRulesAction = 'disable'
            rules = @(@{
                name    = $otherRuleName
                enabled = $true
            })
        } | ConvertTo-Json -Compress -Depth 5

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        # The already-disabled test rule should NOT appear in results (skipped)
        $testEntry = $result.rules | Where-Object { $_.name -eq $testRuleName -and $_._metadata.whatIf -match 'disable' }
        $testEntry | Should -BeNullOrEmpty

        Remove-NetFirewallRule -Name $otherRuleName -ErrorAction SilentlyContinue
    }

    It 'reports would remove unspecified rules when unspecifiedRulesAction is remove' -Skip:(!$isElevated) {
        Initialize-TestFirewallRule

        # Specify a different rule so testRuleName is "unspecified"
        # Use a well-known Windows rule that will exist
        $knownRule = (Get-NetFirewallRule | Select-Object -First 1).Name

        $json = @{
            unspecifiedRulesAction = 'remove'
            rules = @(@{
                name    = $knownRule
                enabled = $true
            })
        } | ConvertTo-Json -Compress -Depth 5

        $result = windows_firewall set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        # The test rule should be among the removed entries
        $removedEntry = $result.rules | Where-Object { $_.name -eq $testRuleName -and $_._metadata.whatIf -match 'Would remove unspecified firewall rule' }
        $removedEntry | Should -Not -BeNullOrEmpty
        $removedEntry._exist | Should -BeFalse

        # Verify no actual removal: the rule still exists
        $actual = Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        $actual | Should -Not -BeNullOrEmpty
    }
}
