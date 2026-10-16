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
        $guests = 'BUILTIN\Guests'
        $anonymous = 'NT AUTHORITY\ANONYMOUS LOGON'

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
            Set-UserRightIdentity -Identity @($guests)
        }

        AfterAll {
            if ($script:originalState) {
                Set-UserRightIdentity -Identity @($script:originalState.identity)
            }
        }

        It 'Projects a grant without applying it' {
            $json = @{ right = $testRight; identity = @($guests, $anonymous) } | ConvertTo-Json -Compress
            $result = windows_security_policy set user-right -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result.right | Should -BeExactly $testRight
            @($result.identity) | Should -Be @($guests, $anonymous)
            $result._metadata.whatIf | Should -Be @("Would grant '$testRight' to '$anonymous'")

            @((Get-UserRightState).identity) | Should -Be @($guests)
        }

        It 'Projects a revoke without applying it' {
            $json = @{ right = $testRight; identity = @() } | ConvertTo-Json -Compress
            $result = windows_security_policy set user-right -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            @($result.identity).Count | Should -Be 0
            $result._metadata.whatIf | Should -Be @("Would revoke '$testRight' from '$guests'")

            @((Get-UserRightState).identity) | Should -Be @($guests)
        }

        It 'Returns no whatIf messages when nothing would change' {
            $json = @{ right = $testRight; identity = @($guests) } | ConvertTo-Json -Compress
            $result = windows_security_policy set user-right -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result._metadata | Should -BeNullOrEmpty
        }

        It 'Fails for an unknown right or account even in what-if mode' {
            $null = windows_security_policy set user-right -w --input '{"right":"SeBogusPrivilege","identity":[]}' 2>$null
            $LASTEXITCODE | Should -Be 2
            $json = @{ right = $testRight; identity = @('NoSuchAccount_zz_12345') } | ConvertTo-Json -Compress
            $null = windows_security_policy set user-right -w --input $json 2>$null
            $LASTEXITCODE | Should -Be 2
        }

        It 'Works through dsc resource set --what-if' {
            $json = @{ right = $testRight; identity = @($anonymous); setAction = 'add' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r Microsoft.Windows/UserRightsAssignment -w -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $out | ConvertFrom-Json
            @($result.afterState.identity) | Should -Be @($guests, $anonymous)
            $result.afterState._metadata.whatIf | Should -Not -BeNullOrEmpty

            @((Get-UserRightState).identity) | Should -Be @($guests)
        }
    }
}
