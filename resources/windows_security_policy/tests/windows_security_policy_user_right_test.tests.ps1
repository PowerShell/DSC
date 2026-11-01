# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'User rights assignment test tests' -Skip:(!$IsWindows) {
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
        $resourceType = 'Microsoft.Windows/UserRightsAssignment'
        $testRight = 'SeDenyBatchLogonRight'
        $guests = 'BUILTIN\Guests'
        $guestsSid = 'S-1-5-32-546'
        $anonymous = 'NT AUTHORITY\ANONYMOUS LOGON'

        function Get-UserRightState {
            $json = @{ right = $testRight } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }

        function Test-UserRightState {
            param([hashtable]$Properties)
            $json = $Properties | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }

        function Set-UserRightIdentity {
            param([string[]]$Identity)
            $json = @{ right = $testRight; identity = @($Identity) } | ConvertTo-Json -Compress
            $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        }
    }

    Context 'Test against a right held by one account' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-UserRightState
            Set-UserRightIdentity -Identity @($guests)
        }

        AfterAll {
            if ($script:originalState) {
                Set-UserRightIdentity -Identity @($script:originalState.identity)
            }
        }

        It 'Replace with the same account is in the desired state' {
            $result = Test-UserRightState -Properties @{ right = $testRight; identity = @($guests) }
            $result.inDesiredState | Should -BeTrue
            $result.actualState._inDesiredState | Should -BeTrue
            @($result.actualState.identity) | Should -Be @($guests)
        }

        It 'Replace compares SIDs, not spellings' {
            (Test-UserRightState -Properties @{ right = $testRight; identity = @($guestsSid) }).inDesiredState | Should -BeTrue
            (Test-UserRightState -Properties @{ right = $testRight; identity = @('guests') }).inDesiredState | Should -BeTrue
        }

        It 'Replace with an extra account is not in the desired state' {
            (Test-UserRightState -Properties @{ right = $testRight; identity = @($guests, $anonymous) }).inDesiredState | Should -BeFalse
        }

        It 'Replace with an empty list is not in the desired state while someone holds the right' {
            (Test-UserRightState -Properties @{ right = $testRight; identity = @() }).inDesiredState | Should -BeFalse
        }

        It 'Add is in the desired state when the accounts already hold the right' {
            (Test-UserRightState -Properties @{ right = $testRight; identity = @($guests); setAction = 'add' }).inDesiredState | Should -BeTrue
            (Test-UserRightState -Properties @{ right = $testRight; identity = @(); setAction = 'add' }).inDesiredState | Should -BeTrue
        }

        It 'Add is not in the desired state when an account is missing' {
            (Test-UserRightState -Properties @{ right = $testRight; identity = @($anonymous); setAction = 'add' }).inDesiredState | Should -BeFalse
        }

        It 'Remove is in the desired state when none of the accounts hold the right' {
            (Test-UserRightState -Properties @{ right = $testRight; identity = @($anonymous); setAction = 'remove' }).inDesiredState | Should -BeTrue
        }

        It 'Remove is not in the desired state when a listed account holds the right' {
            (Test-UserRightState -Properties @{ right = $testRight; identity = @($guests); setAction = 'remove' }).inDesiredState | Should -BeFalse
        }

        It 'Reports no differing properties when in the desired state with <description>' -TestCases @(
            @{ description = 'an explicit replace'; properties = @{ identity = @('BUILTIN\Guests'); setAction = 'replace' } }
            @{ description = 'another spelling of the account'; properties = @{ identity = @('S-1-5-32-546') } }
            @{ description = 'an add of an account that holds the right'; properties = @{ identity = @('BUILTIN\Guests'); setAction = 'add' } }
            @{ description = 'a remove of an account that does not hold the right'; properties = @{ identity = @('NT AUTHORITY\ANONYMOUS LOGON'); setAction = 'remove' } }
        ) {
            $result = Test-UserRightState -Properties (@{ right = $testRight } + $properties)
            $result.inDesiredState | Should -BeTrue
            $result.differingProperties | Should -BeNullOrEmpty
        }

        It 'Reports only identity as differing with setAction <setAction>' -TestCases @(
            @{ setAction = 'replace'; identity = @('NT AUTHORITY\ANONYMOUS LOGON') }
            @{ setAction = 'add'; identity = @('NT AUTHORITY\ANONYMOUS LOGON') }
            @{ setAction = 'remove'; identity = @('BUILTIN\Guests') }
        ) {
            $result = Test-UserRightState -Properties @{ right = $testRight; identity = $identity; setAction = $setAction }
            $result.inDesiredState | Should -BeFalse
            @($result.differingProperties) | Should -Be @('identity')
        }

        It 'Fails when identity is missing' {
            $null = windows_security_policy test user-right --input "{`"right`":`"$testRight`"}" 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 2
        }

        It 'Writes an informational message when <spelling> is not the canonical name' -TestCases @(
            @{ spelling = 'Guests' }
            @{ spelling = 'builtin\guests' }
            @{ spelling = 'S-1-5-32-546' }
            @{ spelling = '*S-1-5-32-546' }
        ) {
            $json = @{ right = $testRight; identity = @($spelling) } | ConvertTo-Json -Compress
            $null = windows_security_policy test user-right --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $info = @(Get-Content $testdrive/error.log | ForEach-Object { ($_ | ConvertFrom-Json).info } | Where-Object { $_ })
            $info.Count | Should -Be 1
            $info[0] | Should -Match ([regex]::Escape("'$spelling' resolves to '$guests'"))
        }

        It 'Writes no informational message for the canonical name' {
            $json = @{ right = $testRight; identity = @($guests) } | ConvertTo-Json -Compress
            $null = windows_security_policy test user-right --input $json 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0
            Get-Content -Raw $testdrive/error.log | Should -BeNullOrEmpty
        }
    }
}
