# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows audit policy export tests' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/AuditPolicy'
        $expectedCategories = @(
            'System'
            'Logon/Logoff'
            'Object Access'
            'Privilege Use'
            'Detailed Tracking'
            'Policy Change'
            'Account Management'
            'DS Access'
            'Account Logon'
        )
    }

    Context 'Export all subcategories' -Skip:(!$isAdmin) {
        BeforeAll {
            $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $script:export = $raw | ConvertFrom-Json
        }

        It 'Returns every subcategory' {
            $script:export.resources.Count | Should -BeGreaterOrEqual 50
        }

        It 'Returns complete, well-formed entries with the correct type' {
            foreach ($resource in $script:export.resources) {
                $resource.type | Should -BeExactly $resourceType
                $resource.properties.subcategory | Should -Not -BeNullOrEmpty
                $resource.properties.guid | Should -Match '^\{[0-9A-F]{8}(-[0-9A-F]{4}){3}-[0-9A-F]{12}\}$'
                $resource.properties.category | Should -Not -BeNullOrEmpty
                $resource.properties.success | Should -BeOfType [bool]
                $resource.properties.failure | Should -BeOfType [bool]
            }
        }

        It 'Uses the canonical English subcategory names' {
            $names = $script:export.resources.properties.subcategory
            $names | Should -Contain 'Logon'
            $names | Should -Contain 'Credential Validation'
            $names | Should -Contain 'Security State Change'
            $logon = $script:export.resources | Where-Object { $_.properties.subcategory -eq 'Logon' }
            $logon.properties.guid | Should -BeExactly '{0CCE9215-69AE-11D9-BED3-505054503030}'
            $logon.properties.category | Should -BeExactly 'Logon/Logoff'
        }

        It 'Covers every category and groups entries by category' {
            $categories = @($script:export.resources.properties.category)
            $unique = @($categories | Select-Object -Unique)
            $unique.Count | Should -Be $expectedCategories.Count
            foreach ($expected in $expectedCategories) {
                $unique | Should -Contain $expected
            }
            # Each category must appear as one contiguous block.
            $blocks = 1
            for ($i = 1; $i -lt $categories.Count; $i++) {
                if ($categories[$i] -ne $categories[$i - 1]) {
                    $blocks++
                }
            }
            $blocks | Should -Be $unique.Count
        }

        It 'Exported entries round-trip through get' {
            foreach ($resource in $script:export.resources | Select-Object -First 3) {
                $json = @{ subcategory = $resource.properties.guid } | ConvertTo-Json -Compress
                $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
                $actual = ($out | ConvertFrom-Json).actualState
                $actual.success | Should -Be $resource.properties.success
                $actual.failure | Should -Be $resource.properties.failure
                $actual.category | Should -BeExactly $resource.properties.category
            }
        }
    }
}
