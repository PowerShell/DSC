# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'User rights assignment get tests' -Skip:(!$IsWindows) {
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

        function Get-UserRightState {
            param([string]$Right)
            $json = @{ right = $Right } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }

        function ConvertTo-SidString {
            param([string]$Identity)
            $text = $Identity.TrimStart('*')
            if ($text -match '^S-1-\d') {
                return $text
            }
            return ([System.Security.Principal.NTAccount]$text).Translate([System.Security.Principal.SecurityIdentifier]).Value
        }

        function Get-SeceditHolders {
            param([string]$Right)
            $inf = Join-Path $testdrive 'user-rights.inf'
            $null = secedit /export /cfg $inf /areas USER_RIGHTS /quiet
            $line = Get-Content $inf | Where-Object { $_ -match "^$Right\s*=" } | Select-Object -First 1
            if (-not $line) {
                return @()
            }
            $values = ($line -split '=', 2)[1].Trim()
            if (-not $values) {
                return @()
            }
            return @($values -split ',' | ForEach-Object { ConvertTo-SidString -Identity $_.Trim() })
        }
    }

    Context 'Get' -Skip:(!$isAdmin) {
        It 'Returns the holders of a right' {
            $result = Get-UserRightState -Right 'SeServiceLogonRight'
            $result.right | Should -BeExactly 'SeServiceLogonRight'
            $result.PSObject.Properties.Name | Should -Contain 'identity'
            foreach ($name in @($result.identity)) {
                $name | Should -Not -BeNullOrEmpty
            }
        }

        It 'Echoes the input casing when called directly' {
            # DSC validates the schema enum, so a lower-case right only reaches the binary directly.
            $out = windows_security_policy get user-right --input '{"right":"seservicelogonright"}' 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            ($out | ConvertFrom-Json).right | Should -BeExactly 'seservicelogonright'
        }

        It 'Returns an empty identity list for a right that no one holds' {
            $result = Get-UserRightState -Right 'SeTrustedCredManAccessPrivilege'
            @($result.identity).Count | Should -Be 0 -Because 'SeTrustedCredManAccessPrivilege is unassigned by default'
        }

        It 'Agrees with secedit for <right>' -TestCases @(
            @{ right = 'SeInteractiveLogonRight' }
            @{ right = 'SeChangeNotifyPrivilege' }
            @{ right = 'SeServiceLogonRight' }
        ) {
            $result = Get-UserRightState -Right $right
            $ours = @(@($result.identity) | ForEach-Object { ConvertTo-SidString -Identity $_ } | Sort-Object)
            $theirs = @(Get-SeceditHolders -Right $right | Sort-Object)
            $ours | Should -Be $theirs
        }
    }

    Context 'Input validation' -Skip:(!$isAdmin) {
        It 'Fails for an unknown right' {
            $null = windows_security_policy get user-right --input '{"right":"SeBogusPrivilege"}' 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 2
            Get-Content -Raw $testdrive/error.log | Should -Match 'Unknown user right'
        }

        It 'Fails when right is missing' {
            $json = @{ identity = @('Administrators') } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when input JSON is invalid' {
            $null = 'not-json' | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Without elevation' -Skip:($isAdmin) {
        It 'Is refused by dsc because the resource requires an elevated process' {
            $json = @{ right = 'SeServiceLogonRight' } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Exits with code 4 when called directly' {
            $null = windows_security_policy get user-right --input '{"right":"SeServiceLogonRight"}' 2>$null
            $LASTEXITCODE | Should -Be 4
        }
    }
}
