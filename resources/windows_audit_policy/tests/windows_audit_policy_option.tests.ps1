# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows audit policy option tests' -Skip:(!$IsWindows) {
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
        $resourceType = 'Microsoft.Windows/AuditPolicyOption'
        $lsaKey = 'HKLM:\SYSTEM\CurrentControlSet\Control\Lsa'

        function Get-RawOptionValue {
            param([string]$Name)
            (Get-ItemProperty -Path $lsaKey -Name $Name -ErrorAction SilentlyContinue).$Name
        }

        function ConvertTo-Enabled {
            param($Raw)
            if ($null -eq $Raw) {
                return $false
            }
            if ($Raw -is [byte[]]) {
                # Like the resource: any non-zero byte means enabled, whatever the length.
                return @($Raw | Where-Object { $_ -ne 0 }).Count -gt 0
            }
            return ($Raw -ne 0)
        }

        function Restore-RawOptionValue {
            param([string]$Name, $Raw)
            if ($null -eq $Raw) {
                Remove-ItemProperty -Path $lsaKey -Name $Name -ErrorAction SilentlyContinue
            }
            else {
                Set-ItemProperty -Path $lsaKey -Name $Name -Value $Raw
            }
        }

        function Get-OptionState {
            param([string]$Name)
            $json = @{ name = $Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }

        function Set-OptionState {
            param([string]$Name, [bool]$Enabled)
            $json = @{ name = $Name; enabled = $Enabled } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json)
        }
    }

    Context 'Get' {
        It 'Returns the <name> option state from the registry' -TestCases @(
            @{ name = 'CrashOnAuditFail' }
            @{ name = 'FullPrivilegeAuditing' }
            @{ name = 'AuditBaseObjects' }
            @{ name = 'AuditBaseDirectories' }
        ) {
            $result = Get-OptionState -Name $name
            $result.name | Should -BeExactly $name
            $result.enabled | Should -BeOfType [bool]
            $result.enabled | Should -Be (ConvertTo-Enabled -Raw (Get-RawOptionValue -Name $name))
        }

        It 'Fails for an unknown option name' {
            $json = @{ name = 'NotAnOption' } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when name is missing' {
            $json = @{ enabled = $true } | ConvertTo-Json -Compress
            $null = $json | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Reports an instance with custom metadata in the desired state' {
            $enabled = if ((Get-OptionState -Name 'AuditBaseObjects').enabled) { 'true' } else { 'false' }
            $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Audit base objects
    type: $resourceType
    metadata:
      owner: DSC-team
    properties:
      name: AuditBaseObjects
      enabled: $enabled
"@
            $configPath = Join-Path $testdrive 'auditpolicyoption-metadata.config.dsc.yaml'
            Set-Content -Path $configPath -Value $yaml

            $out = dsc config test -f $configPath 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).results[0].result
            $result.inDesiredState | Should -BeTrue
            $result.differingProperties | Should -BeNullOrEmpty
        }
    }

    Context 'Export' {
        It 'Returns all four options' {
            $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $raw | ConvertFrom-Json
            $result.resources.Count | Should -Be 4
            foreach ($expected in 'CrashOnAuditFail', 'FullPrivilegeAuditing', 'AuditBaseObjects', 'AuditBaseDirectories') {
                $result.resources.properties.name | Should -Contain $expected
            }
            foreach ($resource in $result.resources) {
                $resource.type | Should -BeExactly $resourceType
                $resource.properties.enabled | Should -BeOfType [bool]
            }
        }
    }

    Context 'Set' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalAuditBaseObjects = Get-RawOptionValue -Name 'AuditBaseObjects'
            $script:originalFullPrivilegeAuditing = Get-RawOptionValue -Name 'FullPrivilegeAuditing'
        }

        AfterAll {
            Restore-RawOptionValue -Name 'AuditBaseObjects' -Raw $script:originalAuditBaseObjects
            Restore-RawOptionValue -Name 'FullPrivilegeAuditing' -Raw $script:originalFullPrivilegeAuditing
        }

        It 'Enables AuditBaseObjects and reports that a restart is required' {
            # Start disabled so the asserted call is the one that writes the change.
            $null = Set-OptionState -Name 'AuditBaseObjects' -Enabled $false
            $result = (Set-OptionState -Name 'AuditBaseObjects' -Enabled $true).afterState
            $result.name | Should -BeExactly 'AuditBaseObjects'
            $result.enabled | Should -BeTrue
            $result._restartRequired.Count | Should -Be 1
            $result._restartRequired[0].system | Should -BeExactly $env:COMPUTERNAME
            Get-RawOptionValue -Name 'AuditBaseObjects' | Should -Be 1
        }

        It 'Disables AuditBaseObjects' {
            $result = (Set-OptionState -Name 'AuditBaseObjects' -Enabled $false).afterState
            $result.enabled | Should -BeFalse
            Get-RawOptionValue -Name 'AuditBaseObjects' | Should -Be 0
        }

        It 'Is idempotent when already in the desired state' {
            # Establish the desired state first so the asserted call is the one that changes nothing.
            $null = Set-OptionState -Name 'AuditBaseObjects' -Enabled $false
            $output = Set-OptionState -Name 'AuditBaseObjects' -Enabled $false
            $output.afterState.enabled | Should -BeFalse
            $output.afterState._restartRequired | Should -BeNullOrEmpty
            $output.changedProperties | Should -BeNullOrEmpty
        }

        It 'Enables FullPrivilegeAuditing and writes it as REG_BINARY' {
            # Start disabled so the asserted call is the one that writes the change.
            $null = Set-OptionState -Name 'FullPrivilegeAuditing' -Enabled $false
            $result = (Set-OptionState -Name 'FullPrivilegeAuditing' -Enabled $true).afterState
            $result.enabled | Should -BeTrue
            $raw = Get-RawOptionValue -Name 'FullPrivilegeAuditing'
            $raw | Should -Not -BeNullOrEmpty
            ConvertTo-Enabled -Raw $raw | Should -BeTrue
            # The LSA reads a REG_BINARY here, so the resource writes that type whatever was there before.
            (Get-Item -Path $lsaKey).GetValueKind('FullPrivilegeAuditing') | Should -Be ([Microsoft.Win32.RegistryValueKind]::Binary)
        }

        It 'Writes AuditBaseObjects as REG_DWORD even when another type was stored' {
            # Some hardening tools store the option as REG_QWORD, which the LSA doesn't read.
            Set-ItemProperty -Path $lsaKey -Name 'AuditBaseObjects' -Value ([uint64]0) -Type QWord
            (Get-Item -Path $lsaKey).GetValueKind('AuditBaseObjects') | Should -Be ([Microsoft.Win32.RegistryValueKind]::QWord)
            $result = (Set-OptionState -Name 'AuditBaseObjects' -Enabled $true).afterState
            $result.enabled | Should -BeTrue
            (Get-Item -Path $lsaKey).GetValueKind('AuditBaseObjects') | Should -Be ([Microsoft.Win32.RegistryValueKind]::DWord)
            Get-RawOptionValue -Name 'AuditBaseObjects' | Should -Be 1
        }

        It 'Fails when enabled is missing' {
            # Called directly: through dsc, the synthetic test would treat the key-only input as
            # already in the desired state and skip set.
            $null = windows_audit_policy set option --input '{"name":"AuditBaseObjects"}' 2>$null
            $LASTEXITCODE | Should -Be 2
        }
    }
}
