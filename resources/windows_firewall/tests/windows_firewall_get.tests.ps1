# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - get operation' -Skip:(!$IsWindows) {
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
        # Skip AppX/UWP rules whose names are ms-resource:// URIs - the COM Item() lookup
        # cannot resolve them by name even though enumeration returns them.
        $knownRule = $exportedRules | Where-Object { $_.name -and $_.name -notmatch 'ms-resource://' } | Select-Object -First 1
        if (-not $knownRule) {
            throw 'No resolvable firewall rule name found on the machine.'
        }
        $knownRuleName = $knownRule.name
    }

    It 'returns an existing rule by name' {
        $json = @{ rules = @(@{ name = $knownRuleName }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).actualState.rules[0]
        $result.name | Should -BeExactly $knownRuleName
        $result.PSObject.Properties.Name | Should -Not -Contain '_exist' -Because ($result | ConvertTo-Json -Depth 10)
        $result.direction | Should -BeIn @('Inbound', 'Outbound')
        $result.action | Should -BeIn @('Allow', 'Block')
    }

    It 'returns an existing rule when name matches' {
        $json = @{ rules = @(@{ name = $knownRuleName }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).actualState.rules[0]
        $result.PSObject.Properties.Name | Should -Not -Contain '_exist' -Because ($result | ConvertTo-Json -Depth 10)
        $result.name | Should -BeExactly $knownRuleName
    }

    It 'returns _exist false with only input properties when the rule is not found' {
        $json = @{ rules = @(@{ name = 'DSC-Missing-FirewallRule'; description = 'input only' }) } | ConvertTo-Json -Compress -Depth 5
        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $result = ($out | ConvertFrom-Json).actualState.rules[0]
        $result.name | Should -BeExactly 'DSC-Missing-FirewallRule'
        $result.description | Should -BeExactly 'input only'
        $result._exist | Should -BeFalse
        $result.PSObject.Properties.Name | Should -Not -Contain 'direction'
    }

    It 'fails when rules array is empty' {
        $json = '{"rules":[]}'
        $out = $json | dsc resource get -r $resourceType -f - 2>&1
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'handles multiple rules in a single request' {
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
