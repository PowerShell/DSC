# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - export operation' -Skip:(!$isElevated) {
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

        function Invoke-DscExport {
            param(
                [string]$InputJson
            )

            if ($InputJson) {
                $raw = dsc resource export -r $resourceType -i $InputJson 2>$testdrive/error.log
            }
            else {
                $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
            }

            return $raw | ConvertFrom-Json
        }

        $initialExport = Invoke-DscExport
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to export firewall rules: $(Get-Content -Raw $testdrive/error.log)"
        }

        $sampleRules = $initialExport.resources[0].properties.rules | Select-Object -First 2 name, direction
        if ($sampleRules.Count -lt 2) {
            throw 'At least two exported firewall rules are required for export tests.'
        }
        $firstRule = $sampleRules[0]
        $secondRule = $sampleRules[1]
    }

    It 'exports all rules with no input' -Skip:(!$isElevated) {
        $output = Invoke-DscExport
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $rules = $output.resources[0].properties.rules
        $rules | Should -Not -BeNullOrEmpty
        $rules.Count | Should -BeGreaterThan 0
        $rules[0].name | Should -Not -BeNullOrEmpty
    }

    It 'applies AND logic within a single filter object' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ name = $firstRule.name; direction = $firstRule.direction }) } | ConvertTo-Json -Compress -Depth 5
        $output = Invoke-DscExport -InputJson $json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $rules = $output.resources[0].properties.rules
        $rules.Count | Should -Be 1
        $rules[0].name | Should -BeExactly $firstRule.name
    }

    It 'applies OR logic across filter objects' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ name = $firstRule.name }, @{ name = $secondRule.name }) } | ConvertTo-Json -Compress -Depth 5
        $output = Invoke-DscExport -InputJson $json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $rules = $output.resources[0].properties.rules
        $names = $rules | ForEach-Object { $_.name }
        $names | Should -Contain $firstRule.name
        $names | Should -Contain $secondRule.name
    }

    It 'supports wildcard name filtering' -Skip:(!$isElevated) {
        # Build a wildcard pattern from the first rule name: take the first word and append '*'
        $prefix = ($firstRule.name -split '[-_ ]')[0]
        $wildcardPattern = "${prefix}*"

        $json = @{ rules = @(@{ name = $wildcardPattern }) } | ConvertTo-Json -Compress -Depth 5
        $output = Invoke-DscExport -InputJson $json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $rules = $output.resources[0].properties.rules
        $rules | Should -Not -BeNullOrEmpty
        $rules | ForEach-Object { $_.name | Should -BeLike $wildcardPattern }
    }

    It 'returns no rules when filter matches nothing' -Skip:(!$isElevated) {
        $json = @{ rules = @(@{ name = 'DSC-NonExistent-Rule-Filter-12345' }) } | ConvertTo-Json -Compress -Depth 5
        $output = Invoke-DscExport -InputJson $json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $rules = $output.resources[0].properties.rules
        $rules.Count | Should -Be 0
    }
}
