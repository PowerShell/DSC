# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
    }
    else {
        $isElevated = (id -u) -eq 0
    }

    $sshdExists = ($null -ne (Get-Command sshd -CommandType Application -ErrorAction Ignore))
    $skipTest = !$isElevated -or !$sshdExists
}

Describe 'SSHDConfig resource tests' -Skip:($skipTest) {
    BeforeAll {
        # set a non-default value in a temporary sshd_config file
        "LogLevel Debug3`nPasswordAuthentication no" | Set-Content -Path $TestDrive/test_sshd_config
        $filepath = Join-Path $TestDrive 'test_sshd_config'
        $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  metadata:
    filepath: $filepath
  properties:
"@
    }

    It '<command> works' -TestCases @(
        @{ command = 'get' }
        @{ command = 'export' }
    ) {
        param($command)
        $out = dsc config $command -i "$yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        if ($command -eq 'export') {
            $out.resources.count | Should -Be 1
            $out.resources[0].properties | Should -Not -BeNullOrEmpty
            $out.resources[0].properties.port | Should -BeNullOrEmpty
            $out.resources[0].properties.passwordAuthentication | Should -Be $false
            $out.resources[0].properties._inheritedDefaults | Should -BeNullOrEmpty
        } else {
            $out.results.count | Should -Be 1
            $out.results.result.actualState | Should -Not -BeNullOrEmpty
            $out.results.result.actualState.port[0] | Should -Be 22
            $out.results.result.actualState.passwordAuthentication | Should -Be $false
            $out.results.result.actualState._inheritedDefaults | Should -Contain 'port'
        }
    }

    It 'Export with filter works' {
        $export_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
    Microsoft.DSC:
        securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  metadata:
    filepath: $filepath
  properties:
    passwordauthentication: 'yes'
"@
        $out = dsc config export -i "$export_yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 1
        ($out.resources[0].properties.psobject.properties | Measure-Object).count | Should -Be 1
        $out.resources[0].properties.passwordAuthentication | Should -Be $false
    }

    It '<command> with _includeDefaults specified works' -TestCases @(
        @{ command = 'get'; includeDefaults = $false }
        @{ command = 'export'; includeDefaults = $true }
    ) {
        param($command, $includeDefaults)
        $filepath = Join-Path $TestDrive 'test_sshd_config'
        $input = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  metadata:
    filepath: $filepath
  properties:
    _includeDefaults: $includeDefaults
"@
        $out = dsc config $command -i "$input" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        if ($command -eq 'export') {
            $out.resources.count | Should -Be 1
            $out.resources[0].properties.loglevel | Should -Be 'debug3'
            $out.resources[0].properties.port | Should -Be 22
            $out.resources[0].properties._inheritedDefaults | Should -BeNullOrEmpty
        } else {
            $out.results.count | Should -Be 1
            ($out.results.result.actualState.psobject.properties | Measure-Object).count | Should -Be 2
            $out.results.result.actualState.loglevel | Should -Be 'debug3'
            $out.results.result.actualState._inheritedDefaults | Should -BeNullOrEmpty
        }
    }

    Context 'Surface a default value that has been set in file' {
        BeforeAll {
            "Port 22" | Set-Content -Path $TestDrive/test_sshd_config
        }

        It '<command> works' -TestCases @(
            @{ command = 'get' }
            @{ command = 'export' }
        ) {
            param($command)
            $out = dsc config $command -i "$yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            if ($command -eq 'export') {
                $out.resources.count | Should -Be 1
                $out.resources[0].properties | Should -Not -BeNullOrEmpty
                $out.resources[0].properties.port[0] | Should -Be 22
                $out.resources[0].properties.passwordauthentication | Should -BeNullOrEmpty
                $out.resources[0].properties._inheritedDefaults | Should -BeNullOrEmpty
            } else {
                $out.results.count | Should -Be 1
                $out.results.result.actualState | Should -Not -BeNullOrEmpty
                $out.results.result.actualState.port | Should -Be 22
                $out.results.result.actualState.passwordAuthentication | Should -Be $true
                $out.results.result.actualState._inheritedDefaults | Should -Not -Contain 'port'
            }
        }
    }

    Context 'Set Commands' {
        It 'Set works with _clobber: true' {
            $set_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  metadata:
    filepath: $filepath
  properties:
    _clobber: true
    port: 1234
    allowUsers:
      - user1
      - user2
    passwordAuthentication: $false
    ciphers:
      - aes128-ctr
      - aes192-ctr
      - aes256-ctr
    addressFamily: inet6
    authorizedKeysFile:
      - ./.ssh/authorized_keys
      - ./.ssh/authorized_keys2
"@
            $out = dsc config set -i "$set_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $out.results.count | Should -Be 1
            $out.results.result.afterState.port | Should -Be 1234
            $out.results.result.afterState.passwordauthentication | Should -Be $false
            $out.results.result.afterState.ciphers | Should -Be @('aes128-ctr', 'aes192-ctr', 'aes256-ctr')
            $out.results.result.afterState.allowusers | Should -Be @('user1', 'user2')
            $out.results.result.afterState.addressfamily | Should -Be 'inet6'
            $out.results.result.afterState.authorizedkeysfile | Should -Be @('./.ssh/authorized_keys', './.ssh/authorized_keys2')
        }
    }
}
