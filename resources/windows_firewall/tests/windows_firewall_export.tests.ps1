# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/FirewallRuleList - export operation' -Skip:(!$IsWindows) {
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
    }

    It 'exports all rules with no input' {
        $output = Invoke-DscExport
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        $rules = $output.resources[0].properties.rules
        $rules | Should -Not -BeNullOrEmpty
        $rules.Count | Should -BeGreaterThan 0
        $rules[0].name | Should -Not -BeNullOrEmpty
    }

    It 'returns an error when export input is provided' {
        $json = @{ rules = @(@{ name = 'DSC-Test-Rule' }) } | ConvertTo-Json -Compress -Depth 5
        Invoke-DscExport -InputJson $json | Out-Null
        $LASTEXITCODE | Should -Be 2
        (Get-Content -Raw $testdrive/error.log) | Should -Match 'does not support export filtering'
    }
}
