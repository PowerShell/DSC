# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows audit policy get tests' -Skip:(!$IsWindows) {
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
        $logonGuid = '{0CCE9215-69AE-11D9-BED3-505054503030}'

        function Get-AuditPolicyState {
            param([string]$Subcategory)
            $json = @{ subcategory = $Subcategory } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }
    }

    Context 'Get by name' -Skip:(!$isAdmin) {
        It 'Returns the policy for a subcategory by English name' {
            $result = Get-AuditPolicyState -Subcategory 'Logon'
            $result.subcategory | Should -BeExactly 'Logon'
            $result.guid | Should -BeExactly $logonGuid
            $result.category | Should -BeExactly 'Logon/Logoff'
            $result.success | Should -BeOfType [bool]
            $result.failure | Should -BeOfType [bool]
        }

        It 'Matches the name case-insensitively and echoes the input' {
            $result = Get-AuditPolicyState -Subcategory 'lOgOn'
            $result.subcategory | Should -BeExactly 'lOgOn'
            $result.guid | Should -BeExactly $logonGuid
        }

        It 'Resolves <subcategory> to category <category>' -TestCases @(
            @{ subcategory = 'Security State Change'; category = 'System' }
            @{ subcategory = 'Logoff'; category = 'Logon/Logoff' }
            @{ subcategory = 'File System'; category = 'Object Access' }
            @{ subcategory = 'Sensitive Privilege Use'; category = 'Privilege Use' }
            @{ subcategory = 'Process Creation'; category = 'Detailed Tracking' }
            @{ subcategory = 'Audit Policy Change'; category = 'Policy Change' }
            @{ subcategory = 'User Account Management'; category = 'Account Management' }
            @{ subcategory = 'Directory Service Access'; category = 'DS Access' }
            @{ subcategory = 'Credential Validation'; category = 'Account Logon' }
        ) {
            $result = Get-AuditPolicyState -Subcategory $subcategory
            $result.subcategory | Should -BeExactly $subcategory
            $result.category | Should -BeExactly $category
            $result.guid | Should -Match '^\{[0-9A-F]{8}(-[0-9A-F]{4}){3}-[0-9A-F]{12}\}$'
        }

        It 'Agrees with auditpol' -Skip:(!$isEnglish) {
            $result = Get-AuditPolicyState -Subcategory 'Logon'
            $row = auditpol /get /subcategory:"Logon" /r | Where-Object { $_ -match ',' } | ConvertFrom-Csv
            $setting = $row.'Inclusion Setting'
            $result.success | Should -Be ($setting -match 'Success')
            $result.failure | Should -Be ($setting -match 'Failure')
        }
    }

    Context 'Get by GUID' -Skip:(!$isAdmin) {
        It 'Accepts a braced GUID in any case and echoes the input' {
            $guidInput = '{0cce9215-69ae-11d9-bed3-505054503030}'
            $result = Get-AuditPolicyState -Subcategory $guidInput
            $result.subcategory | Should -BeExactly $guidInput
            $result.guid | Should -BeExactly $logonGuid
            $result.category | Should -BeExactly 'Logon/Logoff'
        }

        It 'Accepts a bare GUID' {
            $result = Get-AuditPolicyState -Subcategory '0CCE9215-69AE-11D9-BED3-505054503030'
            $result.guid | Should -BeExactly $logonGuid
            $result.category | Should -BeExactly 'Logon/Logoff'
        }
    }

    Context 'Input validation' -Skip:(!$isAdmin) {
        It 'Fails for an unknown subcategory name' {
            $json = @{ subcategory = 'Not A Subcategory' } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $testdrive/error.log | Should -Match 'Unknown audit policy subcategory'
        }

        It 'Fails for a GUID that is not a subcategory' {
            $json = @{ subcategory = '{11111111-2222-3333-4444-555555555555}' } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when subcategory is missing' {
            $json = @{ success = $true } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when input JSON is invalid' {
            $null = 'not-json' | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Without elevation' -Skip:($isAdmin) {
        It 'Fails because the resource requires an elevated process' {
            $json = @{ subcategory = 'Logon' } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
        }
    }
}
