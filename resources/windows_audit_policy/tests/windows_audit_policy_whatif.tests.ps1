# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows audit policy whatif tests' -Skip:(!$IsWindows) {
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
        # 'Other Privilege Use Events' is documented by Microsoft as not currently in use.
        $testSubcategory = 'Other Privilege Use Events'
        $lsaKey = 'HKLM:\SYSTEM\CurrentControlSet\Control\Lsa'

        function Get-AuditPolicyState {
            $json = @{ subcategory = $testSubcategory } | ConvertTo-Json -Compress
            $out = windows_audit_policy get subcategory --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }

        function Get-RawOptionValue {
            param([string]$Name)
            (Get-ItemProperty -Path $lsaKey -Name $Name -ErrorAction SilentlyContinue).$Name
        }
    }

    Context 'Subcategory what-if' -Skip:(!$isAdmin) {
        It 'Projects a success change without applying it' {
            $before = Get-AuditPolicyState
            $desired = -not $before.success
            $json = @{ subcategory = $testSubcategory; success = $desired } | ConvertTo-Json -Compress

            $result = windows_audit_policy set subcategory -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result.subcategory | Should -BeExactly $testSubcategory
            $result.success | Should -Be $desired
            $result.failure | Should -Be $before.failure
            $result.guid | Should -BeExactly $before.guid
            $result.category | Should -BeExactly 'Privilege Use'
            $result._metadata.whatIf | Should -Not -BeNullOrEmpty
            $expected = "Would change success auditing for subcategory '$testSubcategory' from $($before.success.ToString().ToLower()) to $($desired.ToString().ToLower())"
            $result._metadata.whatIf | Should -Contain $expected

            # Assert no mutation happened
            $after = Get-AuditPolicyState
            $after.success | Should -Be $before.success
            $after.failure | Should -Be $before.failure
        }

        It 'Projects changes to both flags' {
            $before = Get-AuditPolicyState
            $json = @{
                subcategory = $testSubcategory
                success     = (-not $before.success)
                failure     = (-not $before.failure)
            } | ConvertTo-Json -Compress

            $result = windows_audit_policy set subcategory -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result._metadata.whatIf.Count | Should -Be 2
            $result._metadata.whatIf[0] | Should -Match '^Would change success auditing'
            $result._metadata.whatIf[1] | Should -Match '^Would change failure auditing'

            $after = Get-AuditPolicyState
            $after.success | Should -Be $before.success
            $after.failure | Should -Be $before.failure
        }

        It 'Returns no whatIf messages when nothing would change' {
            $before = Get-AuditPolicyState
            $json = @{
                subcategory = $testSubcategory
                success     = $before.success
                failure     = $before.failure
            } | ConvertTo-Json -Compress

            $result = windows_audit_policy set subcategory -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result._metadata | Should -BeNullOrEmpty
        }

        It 'Fails for an unknown subcategory' {
            $json = @{ subcategory = 'Bogus Subcategory'; success = $true } | ConvertTo-Json -Compress
            $null = windows_audit_policy set subcategory -w --input $json 2>$null
            $LASTEXITCODE | Should -Be 2
        }

        It 'Works through dsc resource set --what-if' {
            $before = Get-AuditPolicyState
            $desired = -not $before.failure
            $json = @{ subcategory = $testSubcategory; failure = $desired } | ConvertTo-Json -Compress

            $out = $json | dsc resource set -r Microsoft.Windows/AuditPolicy -w -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $out | ConvertFrom-Json
            $result.afterState.failure | Should -Be $desired
            $result.afterState._metadata.whatIf | Should -Not -BeNullOrEmpty

            (Get-AuditPolicyState).failure | Should -Be $before.failure
        }
    }

    Context 'Option what-if' -Skip:(!$isAdmin) {
        It 'Projects an option change without touching the registry' {
            $rawBefore = Get-RawOptionValue -Name 'AuditBaseObjects'
            $current = windows_audit_policy get option --input '{"name":"AuditBaseObjects"}' 2>$null | ConvertFrom-Json
            $desired = -not $current.enabled
            $json = @{ name = 'AuditBaseObjects'; enabled = $desired } | ConvertTo-Json -Compress

            $result = windows_audit_policy set option -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result.name | Should -BeExactly 'AuditBaseObjects'
            $result.enabled | Should -Be $desired
            $result._metadata.whatIf | Should -Match "Would change audit policy option 'AuditBaseObjects'"

            Get-RawOptionValue -Name 'AuditBaseObjects' | Should -Be $rawBefore
        }

        It 'Projects enabling CrashOnAuditFail without applying it' {
            $rawBefore = Get-RawOptionValue -Name 'CrashOnAuditFail'

            $result = windows_audit_policy set option -w --input '{"name":"CrashOnAuditFail","enabled":true}' 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result.enabled | Should -BeTrue
            Get-RawOptionValue -Name 'CrashOnAuditFail' | Should -Be $rawBefore
        }
    }
}
