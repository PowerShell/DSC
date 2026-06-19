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
    $skipSshdTest = !$isElevated -or !$sshdExists
}

Describe 'sshdconfig manifest what-if definitions' {
    It 'Defines what-if for <ManifestName>' -ForEach @(
        @{ ManifestName = 'sshd_config.dsc.resource.json' }
        @{ ManifestName = 'sshd-windows.dsc.resource.json' }
        @{ ManifestName = 'sshd-subsystem.dsc.resource.json' }
        @{ ManifestName = 'sshd-subsystemList.dsc.resource.json' }
    ) {
        $manifest = Get-Content -Raw -Path (Join-Path $PSScriptRoot '..' $ManifestName) | ConvertFrom-Json

        $manifest.set.whatIfReturns | Should -BeExactly 'state'
        $whatIfArg = $manifest.set.args | Where-Object { $_.whatIfArg }
        $whatIfArg.whatIfArg | Should -BeExactly '--what-if'
    }
}

Describe 'sshdconfig what-if set tests' -Skip:($skipSshdTest) {
    BeforeAll {
        $TestDir = Join-Path $TestDrive 'sshd_whatif_test'
        New-Item -Path $TestDir -ItemType Directory -Force | Out-Null
        $TestConfigPath = Join-Path $TestDir 'sshd_config'

        if ($IsWindows) {
            $script:DefaultSftpPath = 'sftp-server.exe'
            $script:AlternatePath = "$env:SystemDrive\OpenSSH\bin\sftp.exe"
        }
        else {
            $script:DefaultSftpPath = '/usr/lib/openssh/sftp-server'
            $script:AlternatePath = '/usr/libexec/sftp-server'
        }
    }

    AfterEach {
        if (Test-Path $TestConfigPath) {
            Remove-Item -Path $TestConfigPath -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path "${TestConfigPath}_backup") {
            Remove-Item -Path "${TestConfigPath}_backup" -Force -ErrorAction SilentlyContinue
        }
    }

    It 'Returns predicted sshd_config state without writing the target file' {
        $inputConfig = @{
            _metadata = @{
                filepath = $TestConfigPath
            }
            _purge = $true
            Port = '1234'
            PasswordAuthentication = $false
        } | ConvertTo-Json

        $output = sshdconfig set --what-if --input $inputConfig -s sshd-config 2>$null
        $LASTEXITCODE | Should -Be 0

        $result = $output | ConvertFrom-Json
        $result.port | Should -Be '1234'
        $result.passwordauthentication | Should -BeFalse
        Test-Path $TestConfigPath | Should -BeFalse
    }

    It 'Returns predicted single subsystem state without updating the file' {
        @"
Port 22
Subsystem sftp $script:DefaultSftpPath
"@ | Set-Content -Path $TestConfigPath

        $inputConfig = @{
            _metadata = @{
                filepath = $TestConfigPath
            }
            _exist = $true
            subsystem = @{
                name = 'newsubsystem'
                value = $script:AlternatePath
            }
        } | ConvertTo-Json

        $output = sshdconfig set --what-if --input $inputConfig -s sshd-config-repeat 2>$null
        $LASTEXITCODE | Should -Be 0

        $result = $output | ConvertFrom-Json
        $result.subsystem.Count | Should -Be 2
        ($result.subsystem | Where-Object { $_.name -ceq 'newsubsystem' }).value | Should -Be $script:AlternatePath
        Get-Content -Raw -Path $TestConfigPath | Should -Not -Match 'newsubsystem'
    }

    It 'Returns predicted subsystem list state without updating the file' {
        @"
Port 22
Subsystem sftp $script:DefaultSftpPath
Subsystem test2 /path/to/test2
"@ | Set-Content -Path $TestConfigPath

        $inputConfig = @{
            _metadata = @{
                filepath = $TestConfigPath
            }
            _purge = $true
            subsystem = @(
                @{
                    name = 'powershell'
                    value = $script:AlternatePath
                }
            )
        } | ConvertTo-Json -Depth 10

        $output = sshdconfig set --what-if --input $inputConfig -s sshd-config-repeat-list 2>$null
        $LASTEXITCODE | Should -Be 0

        $result = $output | ConvertFrom-Json
        $result.subsystem.Count | Should -Be 1
        $result.subsystem[0].name | Should -BeExactly 'powershell'
        Get-Content -Raw -Path $TestConfigPath | Should -Match 'test2'
        Get-Content -Raw -Path $TestConfigPath | Should -Not -Match 'powershell'
    }
}

Describe 'sshdconfig Windows global what-if tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $RegistryPath = 'HKLM:\SOFTWARE\OpenSSH'
        $ValueNames = @('DefaultShell', 'DefaultShellCommandOption', 'DefaultShellEscapeArguments')
        $OriginalValues = @{}

        if (Test-Path $RegistryPath) {
            foreach ($valueName in $ValueNames) {
                $value = Get-ItemProperty -Path $RegistryPath -Name $valueName -ErrorAction SilentlyContinue
                if ($null -ne $value) {
                    $OriginalValues[$valueName] = $value.$valueName
                }
            }
        }
    }

    It 'Returns predicted default shell state without updating registry values' {
        $inputConfig = @{
            shell = 'C:\Windows\System32\cmd.exe'
            cmdOption = '/c'
            escapeArguments = $false
        } | ConvertTo-Json

        $output = sshdconfig set --what-if --input $inputConfig -s windows-global 2>$null
        $LASTEXITCODE | Should -Be 0

        $result = $output | ConvertFrom-Json
        $result.shell | Should -Be 'C:\Windows\System32\cmd.exe'
        $result.cmdOption | Should -Be '/c'
        $result.escapeArguments | Should -BeFalse

        foreach ($valueName in $ValueNames) {
            $value = Get-ItemProperty -Path $RegistryPath -Name $valueName -ErrorAction SilentlyContinue
            if ($OriginalValues.ContainsKey($valueName)) {
                $value.$valueName | Should -Be $OriginalValues[$valueName]
            }
            else {
                $value | Should -BeNullOrEmpty
            }
        }
    }
}