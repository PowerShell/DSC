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
        It 'Set works with _purge: true' {
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
    _purge: true
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

    Context 'Subsystem and SubsystemList Tests' {
        BeforeAll {
            # Create a temporary test directory for sshd_config files
            $TestDir = Join-Path $TestDrive "sshd_test"
            New-Item -Path $TestDir -ItemType Directory -Force | Out-Null
            $script:TestConfigPath = Join-Path $TestDir "sshd_config"

            # Define OS-specific paths with spaces
            if ($IsWindows) {
                $script:PathWithSpaces = "C:\Program Files\OpenSSH\sftp-server.exe"
                $script:DefaultSftpPath = "sftp-server.exe"
                $script:AlternatePath = "C:\OpenSSH\bin\sftp.exe"
            }
            else {
                $script:PathWithSpaces = "/usr/local/lib/openssh server/sftp-server"
                $script:DefaultSftpPath = "/usr/lib/openssh/sftp-server"
                $script:AlternatePath = "/usr/libexec/sftp-server"
            }
        }

        BeforeEach {
            # Create test config with existing subsystems
            $initialContent = @"
Port 22
subsystem sftp $script:DefaultSftpPath
Subsystem test2 /path/to/test2
PasswordAuthentication yes
"@
            Set-Content -Path $script:TestConfigPath -Value $initialContent
        }

        AfterEach {
            # Clean up test config file after each test
            if (Test-Path $script:TestConfigPath) {
                Remove-Item -Path $script:TestConfigPath -Force -ErrorAction Ignore
            }
            if (Test-Path "${script:TestConfigPath}_backup") {
                Remove-Item -Path "${script:TestConfigPath}_backup" -Force -ErrorAction Ignore
            }
        }

        It 'Should add a new subsystem that does not already exist' -Skip:($script:skipSubsystemTests) {
            $config_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: newsub
  type: Microsoft.OpenSSH.SSHD/Subsystem
  metadata:
    filepath: $script:TestConfigPath
  properties:
    _exist: true
    subsystem:
      name: newsubsystem
      value: /path/to/newsubsystem
"@
            $out = dsc config set -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $out.hadErrors | Should -BeFalse
            $out.results.Count | Should -Be 1
            $out.results[0].type | Should -BeExactly 'Microsoft.OpenSSH.SSHD/Subsystem'
            $out.results[0].result.afterState._exist | Should -Be $true
            $out.results[0].result.afterState.subsystem.name | Should -Be 'newsubsystem'

            $getResult = dsc config get -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $getResult.results[0].result.actualState._exist | Should -Be $true
            $getResult.results[0].result.actualState.subsystem.name | Should -Be 'newsubsystem'
            $getResult.results[0].result.actualState.subsystem.value | Should -Be '/path/to/newsubsystem'
        }

        It 'Should remove a subsystem when _exist is false' -Skip:($script:skipSubsystemTests) {
            $config_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: removesub
  type: Microsoft.OpenSSH.SSHD/Subsystem
  metadata:
    filepath: $script:TestConfigPath
  properties:
    _exist: false
    subsystem:
      name: sftp
"@
            $out = dsc config set -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $out.hadErrors | Should -BeFalse
            $out.results[0].result.afterState._exist | Should -Be $false

            $getResult = dsc config get -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $getResult.results[0].result.actualState._exist | Should -Be $false
        }

        It 'Should add multiple new subsystems with SubsystemList' -Skip:($script:skipSubsystemTests) {
            $config_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: multisubsystem
  type: Microsoft.OpenSSH.SSHD/SubsystemList
  metadata:
    filepath: $script:TestConfigPath
  properties:
    _purge: false
    subsystem:
    - name: newsub1
      value: /path/to/newsub1
    - name: newsub2
      value: /path/to/newsub2
"@
            $out = dsc config set -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $out.hadErrors | Should -BeFalse
            $out.results[0].type | Should -BeExactly 'Microsoft.OpenSSH.SSHD/SubsystemList'
            $out.results[0].result.afterState.subsystem.Count | Should -Be 4

            # Verify all subsystems are present (old + new)
            $subsystems = Get-Content $script:TestConfigPath | Where-Object { $_ -match '^\s*subsystem\s+' }
            $subsystems.Count | Should -Be 4
            $subsystems | Should -Contain "subsystem newsub1 /path/to/newsub1"
            $subsystems | Should -Contain "subsystem newsub2 /path/to/newsub2"

            # Verify that new subsystems were appended (not inserted)
            $allLines = Get-Content $script:TestConfigPath
            $newsub1Line = ($allLines | Select-String -Pattern 'subsystem\s+newsub1').LineNumber
            $newsub2Line = ($allLines | Select-String -Pattern 'subsystem\s+newsub2').LineNumber
            $sftpLine = ($allLines | Select-String -Pattern 'subsystem\s+sftp').LineNumber
            $test2Line = ($allLines | Select-String -Pattern 'Subsystem\s+test2').LineNumber

            # New subsystems should be added after the original subsystems
            $newsub1Line | Should -BeGreaterThan $sftpLine
            $newsub1Line | Should -BeGreaterThan $test2Line
            $newsub2Line | Should -BeGreaterThan $sftpLine
            $newsub2Line | Should -BeGreaterThan $test2Line
        }

        It 'Should preserve unlisted subsystems when _purge is false' {
            $config_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: preservesubsystem
  type: Microsoft.OpenSSH.SSHD/SubsystemList
  metadata:
    filepath: $script:TestConfigPath
  properties:
    _purge: false
    subsystem:
    - name: addedSubsystem
      value: /path/to/this
"@
            $out = dsc config set -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $out.hadErrors | Should -BeFalse

            # Verify using dsc config get
            $getResult = dsc config get -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $subsystems = $getResult.results[0].result.actualState.subsystem
            $subsystems.Count | Should -Be 3

            # Verify each subsystem
            foreach ($subsystem in $subsystems) {
                $subsystem.name | Should -BeIn @('addedSubsystem', 'sftp', 'test2')
                if ($subsystem.name -eq 'addedSubsystem') {
                    $subsystem.value | Should -Be '/path/to/this'
                }
            }
        }

        It 'Should remove unlisted subsystems when _purge is true' {
            $config_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: purgesubsystem
  type: Microsoft.OpenSSH.SSHD/SubsystemList
  metadata:
    filepath: $script:TestConfigPath
  properties:
    _purge: true
    subsystem:
    - name: sftp
      value: $script:AlternatePath
    - name: newSub
      value: /path/to/newSub
"@
            $out = dsc config set -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $out.hadErrors | Should -BeFalse

            # Verify using dsc config get
            $getResult = dsc config get -i "$config_yaml" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $subsystems = $getResult.results[0].result.actualState.subsystem
            $subsystems.Count | Should -Be 2

            # Verify each subsystem
            foreach ($subsystem in $subsystems) {
                $subsystem.name | Should -BeIn @('sftp', 'newSub')
                $subsystem.name | Should -Not -Be 'test2'
                if ($subsystem.name -eq 'sftp') {
                    $subsystem.value | Should -Be $script:AlternatePath
                }
                if ($subsystem.name -eq 'newSub') {
                    $subsystem.value | Should -Be '/path/to/newSub'
                }
            }
        }
    }
}
