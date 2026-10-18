# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'User rights assignment set tests' -Skip:(!$IsWindows) {
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
        # SeDenyBatchLogonRight is unassigned by default and denying batch logon to the test
        # accounts below has no effect on a running system.
        $testRight = 'SeDenyBatchLogonRight'
        $guests = 'BUILTIN\Guests'
        $guestsSid = 'S-1-5-32-546'
        $anonymous = 'NT AUTHORITY\ANONYMOUS LOGON'
        $anonymousSid = 'S-1-5-7'

        function Get-UserRightState {
            param([string]$Right = $testRight)
            $json = @{ right = $Right } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }

        function Set-UserRightState {
            param([hashtable]$Properties)
            $json = $Properties | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }

        function Restore-UserRightState {
            param($Original)
            if ($Original) {
                $json = @{ right = $testRight; identity = @($Original.identity) } | ConvertTo-Json -Compress
                $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            }
        }

        function Get-SeceditLine {
            param([string]$Right)
            $inf = Join-Path $testdrive 'user-rights.inf'
            $null = secedit /export /cfg $inf /areas USER_RIGHTS /quiet
            return (Get-Content $inf | Where-Object { $_ -match "^$Right\s*=" } | Select-Object -First 1)
        }
    }

    Context 'Set holders' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-UserRightState
        }

        BeforeEach {
            $null = Set-UserRightState -Properties @{ right = $testRight; identity = @($guests) }
        }

        AfterAll {
            Restore-UserRightState -Original $script:originalState
        }

        It 'Replaces the holders with the listed account' {
            $result = (Set-UserRightState -Properties @{ right = $testRight; identity = @($anonymous) }).afterState
            $result.right | Should -BeExactly $testRight
            @($result.identity) | Should -Be @($anonymous)
            $line = Get-SeceditLine -Right $testRight
            $line | Should -Match ([regex]::Escape("*$anonymousSid"))
            $line | Should -Not -Match ([regex]::Escape("*$guestsSid"))
        }

        It 'Resolves a SID to the canonical name and is idempotent' {
            $output = Set-UserRightState -Properties @{ right = $testRight; identity = @($guestsSid) }
            @($output.afterState.identity) | Should -Be @($guests)
            $output.changedProperties | Should -BeNullOrEmpty
        }

        It 'Accepts the secedit SID form' {
            $output = Set-UserRightState -Properties @{ right = $testRight; identity = @("*$guestsSid") }
            @($output.afterState.identity) | Should -Be @($guests)
            $output.changedProperties | Should -BeNullOrEmpty
        }

        It 'Adds an account without removing the existing holders' {
            $result = (Set-UserRightState -Properties @{ right = $testRight; identity = @($anonymous); setAction = 'add' }).afterState
            @($result.identity) | Should -Be @($guests, $anonymous)
        }

        It 'Removes only the listed holders' {
            $null = Set-UserRightState -Properties @{ right = $testRight; identity = @($guests, $anonymous) }
            $result = (Set-UserRightState -Properties @{ right = $testRight; identity = @($guests); setAction = 'remove' }).afterState
            @($result.identity) | Should -Be @($anonymous)
        }

        It 'Replaces with an empty list to revoke everyone' {
            $result = (Set-UserRightState -Properties @{ right = $testRight; identity = @() }).afterState
            @($result.identity).Count | Should -Be 0
            $line = Get-SeceditLine -Right $testRight
            if ($line) {
                ($line -split '=', 2)[1].Trim() | Should -BeNullOrEmpty
            }
        }

        It 'Collapses aliases of the same account' {
            $result = (Set-UserRightState -Properties @{ right = $testRight; identity = @('Guests', $guests, $guestsSid) }).afterState
            @($result.identity) | Should -Be @($guests)
        }
    }

    Context 'Input validation' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-UserRightState
            $null = Set-UserRightState -Properties @{ right = $testRight; identity = @($guests) }
        }

        AfterAll {
            Restore-UserRightState -Original $script:originalState
        }

        It 'Fails when identity is missing' {
            # Called directly: through dsc, the test operation would run first and reject the input.
            $null = windows_security_policy set user-right --input "{`"right`":`"$testRight`"}" 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $testdrive/error.log | Should -Match "'identity' is required"
        }

        It 'Rejects an unknown option <position> --input without changing the policy' -TestCases @(
            @{ position = 'before' }
            @{ position = 'after' }
        ) {
            $json = @{ right = $testRight; identity = @($anonymous) } | ConvertTo-Json -Compress
            $arguments = if ($position -eq 'before') { @('--whatif', '--input', $json) } else { @('--input', $json, '--whatif') }
            $null = windows_security_policy set user-right @arguments 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 1
            Get-Content -Raw $testdrive/error.log | Should -Match "Unknown option: '--whatif'"
            @((Get-UserRightState).identity) | Should -Be @($guests)
        }

        It 'Fails for an unknown account' {
            $json = @{ right = $testRight; identity = @('NoSuchAccount_zz_12345') } | ConvertTo-Json -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $testdrive/error.log | Should -Match 'could not be found'
        }

        It 'Fails for a malformed SID' {
            $json = @{ right = $testRight; identity = @('S-1-5-32-abc') } | ConvertTo-Json -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $testdrive/error.log | Should -Match 'not a valid SID'
        }

        It 'Rejects an embedded null character in a <kind> without changing the policy' -TestCases @(
            @{ kind = 'SID'; identity = "S-1-5-32-546`0suffix" }
            @{ kind = 'account name'; identity = "Guests`0suffix" }
        ) {
            $json = @{ right = $testRight; identity = @($identity, $anonymous) } | ConvertTo-Json -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $testdrive/error.log | Should -Match 'embedded null character'
            @((Get-UserRightState).identity) | Should -Be @($guests)
        }

        It 'Fails when input JSON is invalid' {
            $null = 'not-json' | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Configuration document' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-UserRightState
            $null = Set-UserRightState -Properties @{ right = $testRight; identity = @() }
            $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Deny batch logon to guests
    type: Microsoft.Windows/UserRightsAssignment
    properties:
      right: $testRight
      identity:
        - $guests
"@
            $script:configPath = Join-Path $testdrive 'userrights.config.dsc.yaml'
            Set-Content -Path $script:configPath -Value $yaml
        }

        AfterAll {
            Restore-UserRightState -Original $script:originalState
        }

        It 'Applies the configuration and then reports it in the desired state' {
            $out = dsc config set -f $script:configPath 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).results[0].result.afterState
            @($result.identity) | Should -Be @($guests)

            $out = dsc config test -f $script:configPath 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            ($out | ConvertFrom-Json).results[0].result.inDesiredState | Should -BeTrue
        }
    }
}
