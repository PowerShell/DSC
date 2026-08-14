# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - synthetic test with schema defaults' -Skip:(!$canRunFirewallTests) {
    BeforeDiscovery {
        $canRunFirewallTests = $IsWindows -and
            (Get-Command Get-NetFirewallRule -ErrorAction Ignore) -and
            ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
                [Security.Principal.WindowsBuiltInRole]::Administrator)
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/FirewallRuleList'
        $testRuleName = 'DSC-WindowsFirewall-SchemaDefault-Test'

        # Ensure a known rule exists for testing
        $existing = Get-NetFirewallRule -Name $testRuleName -ErrorAction Ignore
        if (-not $existing) {
            New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName `
                -Direction Inbound -Action Allow -Protocol TCP -LocalPort 32921 `
                -Enabled True -PolicyStore PersistentStore | Out-Null
        }
    }

    AfterAll {
        Remove-NetFirewallRule -Name $testRuleName -ErrorAction Ignore
    }

    It 'unspecifiedRules action "ignore" does not report as differing' {
        $json = @{
            unspecifiedRules = @{
                action = 'ignore'
            }
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
        $result.differingProperties | Should -Not -Contain 'unspecifiedRules'
    }

    It 'unspecifiedRules omitted does not report as differing' {
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
        $result.differingProperties | Should -Not -Contain 'unspecifiedRules'
    }

    It 'unspecifiedRules action "disable" is ignored for comparison' {
        $json = @{
            unspecifiedRules = @{
                action = 'disable'
            }
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
        $result.differingProperties | Should -Not -Contain 'unspecifiedRules'
    }

    It 'unspecifiedRules action "remove" is ignored for comparison' {
        $json = @{
            unspecifiedRules = @{
                action = 'remove'
            }
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
        $result.differingProperties | Should -Not -Contain 'unspecifiedRules'
    }
}
