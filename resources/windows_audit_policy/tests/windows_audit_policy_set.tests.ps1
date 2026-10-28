# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows audit policy set tests' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
        # auditpol localizes its output with the display (UI) language, not the regional format.
        $isEnglish = (Get-UICulture).Name -like 'en-*'
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/AuditPolicy'
        # 'Other Privilege Use Events' is documented by Microsoft as not currently in use, so
        # toggling its auditing generates no events and is safe to change briefly.
        $testSubcategory = 'Other Privilege Use Events'
        $testGuid = '{0CCE922A-69AE-11D9-BED3-505054503030}'

        function Get-AuditPolicyState {
            param([string]$Subcategory = $testSubcategory)
            $json = @{ subcategory = $Subcategory } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }

        function Set-AuditPolicyState {
            param([hashtable]$Properties)
            $json = $Properties | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }

        function Restore-AuditPolicyState {
            param($Original)
            if ($Original) {
                $json = @{
                    subcategory = $testSubcategory
                    success     = $Original.success
                    failure     = $Original.failure
                } | ConvertTo-Json -Compress
                $null = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            }
        }
    }

    Context 'Set success and failure flags' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-AuditPolicyState
        }

        AfterAll {
            Restore-AuditPolicyState -Original $script:originalState
        }

        It 'Enables success auditing and leaves failure unchanged' {
            $before = Get-AuditPolicyState
            $result = (Set-AuditPolicyState -Properties @{ subcategory = $testSubcategory; success = $true }).afterState
            $result.subcategory | Should -BeExactly $testSubcategory
            $result.success | Should -BeTrue
            $result.failure | Should -Be $before.failure
            $result.guid | Should -BeExactly $testGuid
            $result.category | Should -BeExactly 'Privilege Use'
        }

        It 'Enables failure auditing and leaves success unchanged' {
            $before = Get-AuditPolicyState
            $result = (Set-AuditPolicyState -Properties @{ subcategory = $testSubcategory; failure = $true }).afterState
            $result.success | Should -Be $before.success
            $result.failure | Should -BeTrue
        }

        It 'Disables both flags' {
            $result = (Set-AuditPolicyState -Properties @{ subcategory = $testSubcategory; success = $false; failure = $false }).afterState
            $result.success | Should -BeFalse
            $result.failure | Should -BeFalse
        }

        It 'Reports no auditing to auditpol after disabling both flags' -Skip:(!$isEnglish) {
            $null = Set-AuditPolicyState -Properties @{ subcategory = $testSubcategory; success = $false; failure = $false }
            $row = auditpol /get /subcategory:"$testSubcategory" /r | Where-Object { $_ -match ',' } | ConvertFrom-Csv
            $row.'Inclusion Setting' | Should -BeExactly 'No Auditing'
        }

        It 'Is idempotent when already in the desired state' {
            # Establish the desired state first so the asserted call is the one that changes nothing.
            $null = Set-AuditPolicyState -Properties @{ subcategory = $testSubcategory; success = $false; failure = $false }
            $output = Set-AuditPolicyState -Properties @{ subcategory = $testSubcategory; success = $false; failure = $false }
            $output.afterState.success | Should -BeFalse
            $output.afterState.failure | Should -BeFalse
            $output.changedProperties | Should -BeNullOrEmpty
        }

        It 'Can set by GUID and echoes the GUID' {
            $result = (Set-AuditPolicyState -Properties @{ subcategory = $testGuid; success = $true }).afterState
            $result.subcategory | Should -BeExactly $testGuid
            $result.success | Should -BeTrue
            (Get-AuditPolicyState).success | Should -BeTrue
        }

        It 'Changes nothing when only the key is provided' {
            $before = Get-AuditPolicyState
            $result = (Set-AuditPolicyState -Properties @{ subcategory = $testSubcategory }).afterState
            $result.success | Should -Be $before.success
            $result.failure | Should -Be $before.failure
        }
    }

    Context 'Input validation' -Skip:(!$isAdmin) {
        It 'Fails when input JSON is invalid' {
            $null = 'not-json' | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails for an unknown subcategory' {
            $json = @{ subcategory = 'Not A Subcategory'; success = $true } | ConvertTo-Json -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $testdrive/error.log | Should -Match 'Unknown audit policy subcategory'
        }
    }

    Context 'Configuration document' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-AuditPolicyState
            $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Audit other privilege use events
    type: Microsoft.Windows/AuditPolicy
    properties:
      subcategory: $testSubcategory
      success: true
      failure: true
"@
            $script:configPath = Join-Path $testdrive 'auditpolicy.config.dsc.yaml'
            Set-Content -Path $script:configPath -Value $yaml
        }

        AfterAll {
            Restore-AuditPolicyState -Original $script:originalState
        }

        It 'Applies the configuration and then reports it in the desired state' {
            $out = dsc config set -f $script:configPath 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).results[0].result.afterState
            $result.subcategory | Should -BeExactly $testSubcategory
            $result.success | Should -BeTrue
            $result.failure | Should -BeTrue

            $out = dsc config test -f $script:configPath 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            ($out | ConvertFrom-Json).results[0].result.inDesiredState | Should -BeTrue
        }
    }
}
