# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - get operation' -Skip:(!$IsWindows) {
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

        $exportRaw = dsc resource export -r $resourceType 2>$testdrive/error.log
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to export firewall rules: $(Get-Content -Raw $testdrive/error.log)"
        }
        $exportedRules = ($exportRaw | ConvertFrom-Json).resources[0].properties.rules
        if (-not $exportedRules -or $exportedRules.Count -eq 0) {
            throw 'No firewall rules were found on the machine.'
        }
        $knownRuleName = $exportedRules[0].name
        if (-not $knownRuleName) {
            throw 'The first exported firewall rule has a null or empty name.'
        }
    }

    It 'returns an existing rule by name' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ name = $knownRuleName }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).actualState.rules[0]
        $result.name | Should -BeExactly $knownRuleName
        $result.PSObject.Properties.Name | Should -Not -Contain '_exist'
        $result.direction | Should -BeIn @('Inbound', 'Outbound')
        $result.action | Should -BeIn @('Allow', 'Block')
    }

    It 'returns an existing rule when name matches' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ name = $knownRuleName }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).actualState.rules[0]
        $result.PSObject.Properties.Name | Should -Not -Contain '_exist'
        $result.name | Should -BeExactly $knownRuleName
    }

    It 'returns _exist false with only input properties when the rule is not found' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ name = 'DSC-Missing-FirewallRule'; description = 'input only' }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).actualState.rules[0]
        $result.name | Should -BeExactly 'DSC-Missing-FirewallRule'
        $result.description | Should -BeExactly 'input only'
        $result._exist | Should -BeFalse
        $result.PSObject.Properties.Name | Should -Not -Contain 'direction'
    }

    It 'fails when rules array is empty' -Skip:(!$isElevated) {
        $json = '{"rules":[]}'
        $out = $json | dsc resource get -r $resourceType -f - 2>&1
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'handles multiple rules in a single request' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ name = $knownRuleName }, @{ name = 'DSC-Missing-FirewallRule' }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $results = ($out | ConvertFrom-Json).actualState.rules
        $results.Count | Should -Be 2
        $results[0].name | Should -BeExactly $knownRuleName
        $results[1].name | Should -BeExactly 'DSC-Missing-FirewallRule'
        $results[1]._exist | Should -BeFalse
    }
}
