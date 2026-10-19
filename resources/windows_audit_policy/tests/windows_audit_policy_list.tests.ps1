# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows audit policy list tests' -Skip:(!$IsWindows) {
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
        $resourceType = 'Microsoft.Windows/AuditPolicyList'
        # 'Other Privilege Use Events' is documented by Microsoft as not currently in use, and the
        # DS Access subcategories only generate events on domain controllers, so toggling their
        # auditing briefly is safe.
        $first = 'Other Privilege Use Events'
        $firstGuid = '{0CCE922A-69AE-11D9-BED3-505054503030}'
        $second = 'Detailed Directory Service Replication'
        $secondGuid = '{0CCE923E-69AE-11D9-BED3-505054503030}'

        function Invoke-ListResource {
            param([string]$Operation, [hashtable]$Properties, [switch]$WhatIf)
            $json = $Properties | ConvertTo-Json -Compress -Depth 5
            $arguments = @('resource', $Operation, '-r', $resourceType, '-f', '-')
            if ($WhatIf) {
                $arguments += '-w'
            }
            $out = $json | dsc @arguments 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }

        function Get-Flags {
            $json = @{ policies = @(@{ subcategory = $first }, @{ subcategory = $second }) } | ConvertTo-Json -Compress -Depth 5
            $out = windows_audit_policy get subcategory-list --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).policies
        }

        function Set-Flags {
            param([bool]$Success, [bool]$Failure)
            $json = @{
                policies = @(
                    @{ subcategory = $first; success = $Success; failure = $Failure }
                    @{ subcategory = $second; success = $Success; failure = $Failure }
                )
            } | ConvertTo-Json -Compress -Depth 5
            $null = windows_audit_policy set subcategory-list --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        }

        function Restore-Flags {
            param($Original)
            if ($Original) {
                $json = @{
                    policies = @($Original | ForEach-Object {
                        @{ subcategory = $_.subcategory; success = $_.success; failure = $_.failure }
                    })
                } | ConvertTo-Json -Compress -Depth 5
                $null = windows_audit_policy set subcategory-list --input $json 2>$testdrive/error.log
            }
        }
    }

    Context 'Get, set and test' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-Flags
        }

        BeforeEach {
            Set-Flags -Success $false -Failure $false
        }

        AfterAll {
            Restore-Flags -Original $script:originalState
        }

        It 'Gets every listed subcategory in the order given and echoes each name as given' {
            $result = Invoke-ListResource -Operation get -Properties @{
                policies = @(@{ subcategory = $secondGuid }, @{ subcategory = $first.ToUpper() })
            }
            $policies = $result.actualState.policies
            $policies.Count | Should -Be 2
            $policies[0].subcategory | Should -BeExactly $secondGuid
            $policies[0].guid | Should -BeExactly $secondGuid
            $policies[0].category | Should -BeExactly 'DS Access'
            $policies[1].subcategory | Should -BeExactly $first.ToUpper()
            $policies[1].guid | Should -BeExactly $firstGuid
            $policies[1].category | Should -BeExactly 'Privilege Use'
            $policies | ForEach-Object { $_.success | Should -BeFalse; $_.failure | Should -BeFalse }
        }

        It 'Sets every listed subcategory and leaves omitted flags unchanged' {
            $result = Invoke-ListResource -Operation set -Properties @{
                policies = @(
                    @{ subcategory = $first; success = $true }
                    @{ subcategory = $second; failure = $true }
                )
            }
            $after = $result.afterState.policies
            $after[0].success | Should -BeTrue
            $after[0].failure | Should -BeFalse
            $after[1].success | Should -BeFalse
            $after[1].failure | Should -BeTrue
            $result.changedProperties | Should -Be @('policies')

            $current = Get-Flags
            $current[0].success | Should -BeTrue
            $current[1].failure | Should -BeTrue
        }

        It 'Is idempotent when every subcategory is already in the desired state' {
            $result = Invoke-ListResource -Operation set -Properties @{
                policies = @(
                    @{ subcategory = $first; success = $false; failure = $false }
                    @{ subcategory = $second; success = $false }
                )
            }
            $result.changedProperties | Should -BeNullOrEmpty
        }

        It 'Is in the desired state only when every entry matches' {
            $matching = @{ policies = @(@{ subcategory = $first; success = $false }, @{ subcategory = $second; failure = $false }) }
            (Invoke-ListResource -Operation test -Properties $matching).inDesiredState | Should -BeTrue

            $oneOff = @{ policies = @(@{ subcategory = $first; success = $false }, @{ subcategory = $second; failure = $true }) }
            (Invoke-ListResource -Operation test -Properties $oneOff).inDesiredState | Should -BeFalse
        }

        It 'Applies a configuration document and then reports it in the desired state' {
            $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Audit baseline
    type: $resourceType
    properties:
      policies:
        - subcategory: $first
          success: true
          failure: true
        - subcategory: $second
          failure: true
"@
            $configPath = Join-Path $testdrive 'auditpolicylist.config.dsc.yaml'
            Set-Content -Path $configPath -Value $yaml

            $out = dsc config set -f $configPath 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $after = ($out | ConvertFrom-Json).results[0].result.afterState.policies
            $after[0].success | Should -BeTrue
            $after[0].failure | Should -BeTrue
            $after[1].failure | Should -BeTrue

            $out = dsc config test -f $configPath 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            ($out | ConvertFrom-Json).results[0].result.inDesiredState | Should -BeTrue
        }
    }

    Context 'What-if' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-Flags
        }

        BeforeEach {
            Set-Flags -Success $false -Failure $false
        }

        AfterAll {
            Restore-Flags -Original $script:originalState
        }

        It 'Projects every entry and describes each change without applying it' {
            $json = @{
                policies = @(
                    @{ subcategory = $first; success = $true; failure = $true }
                    @{ subcategory = $second; success = $false }
                )
            } | ConvertTo-Json -Compress -Depth 5
            $out = windows_audit_policy set subcategory-list -w --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $out | ConvertFrom-Json

            $result.policies[0].success | Should -BeTrue
            $result.policies[0].failure | Should -BeTrue
            $result.policies[1].success | Should -BeFalse
            $result.policies[1].failure | Should -BeFalse
            $result._metadata.whatIf | Should -Be @(
                "Would change success auditing for subcategory '$first' from false to true"
                "Would change failure auditing for subcategory '$first' from false to true"
            )

            Get-Flags | ForEach-Object { $_.success | Should -BeFalse; $_.failure | Should -BeFalse }
        }

        It 'Returns no whatIf messages when nothing would change' {
            $json = @{ policies = @(@{ subcategory = $first; success = $false }, @{ subcategory = $second }) } | ConvertTo-Json -Compress -Depth 5
            $result = windows_audit_policy set subcategory-list -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result._metadata | Should -BeNullOrEmpty
        }

        It 'Works through dsc resource set --what-if' {
            $result = Invoke-ListResource -Operation set -WhatIf -Properties @{
                policies = @(@{ subcategory = $second; failure = $true })
            }
            $result.afterState.policies[0].failure | Should -BeTrue
            $result.afterState._metadata.whatIf | Should -Be @(
                "Would change failure auditing for subcategory '$second' from false to true"
            )

            (Get-Flags)[1].failure | Should -BeFalse
        }
    }

    Context 'Input validation' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-Flags
        }

        BeforeEach {
            Set-Flags -Success $false -Failure $false
        }

        AfterAll {
            Restore-Flags -Original $script:originalState
        }

        It 'Rejects a subcategory listed twice as <spelling> without changing anything' -TestCases @(
            @{ spelling = 'Other Privilege Use Events' }
            @{ spelling = 'other privilege use events' }
            @{ spelling = '{0CCE922A-69AE-11D9-BED3-505054503030}' }
        ) {
            $json = @{
                policies = @(
                    @{ subcategory = $first; success = $true }
                    @{ subcategory = $spelling; failure = $true }
                )
            } | ConvertTo-Json -Compress -Depth 5
            $null = windows_audit_policy set subcategory-list --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $testdrive/error.log | Should -Match 'listed more than once'
            Get-Flags | ForEach-Object { $_.success | Should -BeFalse; $_.failure | Should -BeFalse }
        }

        It 'Changes nothing when any entry names an unknown subcategory' {
            $json = @{
                policies = @(
                    @{ subcategory = $first; success = $true }
                    @{ subcategory = 'Not A Subcategory'; success = $true }
                )
            } | ConvertTo-Json -Compress -Depth 5
            $null = windows_audit_policy set subcategory-list --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $testdrive/error.log | Should -Match 'Unknown audit policy subcategory'
            (Get-Flags)[0].success | Should -BeFalse
        }

        It 'Is refused by the resource schema for an empty list' {
            $null = '{"policies":[]}' | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $testdrive/error.log | Should -Match 'less than 1 item'
        }
    }

    Context 'Export' -Skip:(!$isAdmin) {
        It 'Exports every subcategory as a single list instance' {
            $out = dsc resource export -r $resourceType 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $resources = @(($out | ConvertFrom-Json).resources)
            $resources.Count | Should -Be 1
            $resources[0].type | Should -BeExactly $resourceType

            $policies = $resources[0].properties.policies
            $single = @(windows_audit_policy export subcategory 2>$null | ConvertFrom-Json)
            $policies.Count | Should -Be $single.Count
            $policies.subcategory | Should -Contain $first
            $policies.subcategory | Should -Contain $second
            $policies | ForEach-Object { $_.guid | Should -Match '^\{[0-9A-F-]{36}\}$' }
        }
    }

    Context 'Elevation' -Skip:($isAdmin) {
        It 'Is refused by dsc because the resource requires an elevated process' {
            $null = '{"policies":[{"subcategory":"Logon"}]}' | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $testdrive/error.log | Should -Match 'elevated'
        }

        It 'Fails with exit code 4 when invoked directly' {
            $null = windows_audit_policy get subcategory-list --input '{"policies":[{"subcategory":"Logon"}]}' 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 4
        }
    }
}
