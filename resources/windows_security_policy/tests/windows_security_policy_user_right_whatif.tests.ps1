# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'User rights assignment whatif tests' -Skip:(!$IsWindows) {
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
        $testRight = 'SeDenyBatchLogonRight'
        $testRightName = 'Deny log on as a batch job'
        $guests = 'BUILTIN\Guests'
        $anonymous = 'NT AUTHORITY\ANONYMOUS LOGON'

        function Invoke-WhatIf {
            param([hashtable]$Properties)
            $json = $Properties | ConvertTo-Json -Compress
            $out = windows_security_policy set user-right -w --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }

        function Get-UserRightState {
            $out = windows_security_policy get user-right --input "{`"right`":`"$testRight`"}" 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }

        function Set-UserRightIdentity {
            param([string[]]$Identity)
            $json = @{ right = $testRight; identity = @($Identity) } | ConvertTo-Json -Compress
            $null = windows_security_policy set user-right --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        }
    }

    Context 'What-if' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-UserRightState
        }

        BeforeEach {
            Set-UserRightIdentity -Identity @($guests)
        }

        AfterAll {
            if ($script:originalState) {
                Set-UserRightIdentity -Identity @($script:originalState.identity)
            }
        }

        It 'Projects a grant without applying it' {
            $result = Invoke-WhatIf -Properties @{ right = $testRight; identity = @($guests, $anonymous) }

            $result.right | Should -BeExactly $testRight
            @($result.identity) | Should -Be @($guests, $anonymous)
            $result._metadata.whatIf | Should -Be @("Would grant '$testRight' ($testRightName) to '$anonymous'")

            @((Get-UserRightState).identity) | Should -Be @($guests)
        }

        It 'Projects a revoke without applying it' {
            $result = Invoke-WhatIf -Properties @{ right = $testRight; identity = @() }

            @($result.identity).Count | Should -Be 0
            $result._metadata.whatIf | Should -Be @("Would revoke '$testRight' ($testRightName) from '$guests'")

            @((Get-UserRightState).identity) | Should -Be @($guests)
        }

        It 'Projects an add that keeps the other holders' {
            $result = Invoke-WhatIf -Properties @{ right = $testRight; identity = @($anonymous); setAction = 'add' }

            @($result.identity) | Should -Be @($guests, $anonymous)
            $result._metadata.whatIf | Should -Be @("Would grant '$testRight' ($testRightName) to '$anonymous'")

            @((Get-UserRightState).identity) | Should -Be @($guests)
        }

        It 'Projects a remove that keeps the other holders' {
            Set-UserRightIdentity -Identity @($guests, $anonymous)

            $result = Invoke-WhatIf -Properties @{ right = $testRight; identity = @($guests); setAction = 'remove' }

            @($result.identity) | Should -Be @($anonymous)
            $result._metadata.whatIf | Should -Be @("Would revoke '$testRight' ($testRightName) from '$guests'")

            @((Get-UserRightState).identity) | Should -Be @($guests, $anonymous)
        }

        It 'Returns the current holders and no whatIf messages when <setAction> would change nothing' -TestCases @(
            @{ setAction = 'replace'; identity = @('BUILTIN\Guests') }
            @{ setAction = 'add'; identity = @('BUILTIN\Guests') }
            @{ setAction = 'remove'; identity = @('NT AUTHORITY\ANONYMOUS LOGON') }
        ) {
            $result = Invoke-WhatIf -Properties @{ right = $testRight; identity = $identity; setAction = $setAction }
            @($result.identity) | Should -Be @($guests)
            $result._metadata | Should -BeNullOrEmpty
        }

        It 'Fails for an unknown right or account even in what-if mode' {
            $null = windows_security_policy set user-right -w --input '{"right":"SeBogusPrivilege","identity":[]}' 2>$null
            $LASTEXITCODE | Should -Be 2
            $json = @{ right = $testRight; identity = @('NoSuchAccount_zz_12345') } | ConvertTo-Json -Compress
            $null = windows_security_policy set user-right -w --input $json 2>$null
            $LASTEXITCODE | Should -Be 2
        }

        It 'Works through dsc resource set --what-if with setAction <setAction>' -TestCases @(
            @{
                setAction = 'add'
                holders   = @('BUILTIN\Guests')
                identity  = @('NT AUTHORITY\ANONYMOUS LOGON')
                expected  = @('BUILTIN\Guests', 'NT AUTHORITY\ANONYMOUS LOGON')
            }
            @{
                setAction = 'remove'
                holders   = @('BUILTIN\Guests', 'NT AUTHORITY\ANONYMOUS LOGON')
                identity  = @('BUILTIN\Guests')
                expected  = @('NT AUTHORITY\ANONYMOUS LOGON')
            }
        ) {
            Set-UserRightIdentity -Identity $holders

            $json = @{ right = $testRight; identity = $identity; setAction = $setAction } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r Microsoft.Windows/UserRightsAssignment -w -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $out | ConvertFrom-Json
            @($result.beforeState.identity) | Should -Be $holders
            @($result.afterState.identity) | Should -Be $expected
            $result.afterState._metadata.whatIf | Should -Not -BeNullOrEmpty

            @((Get-UserRightState).identity) | Should -Be $holders
        }
    }
}
