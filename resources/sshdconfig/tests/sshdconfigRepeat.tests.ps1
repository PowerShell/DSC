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

Describe 'sshd-config-repeat Set Tests' -Skip:($skipTest) {
    BeforeAll {
        # Create a temporary test directory for sshd_config files
        $TestDir = Join-Path $TestDrive "sshd_test"
        New-Item -Path $TestDir -ItemType Directory -Force | Out-Null
        $TestConfigPath = Join-Path $TestDir "sshd_config"

        # Define OS-specific paths with spaces
        if ($IsWindows) {
            $script:PathWithSpaces = "$env:ProgramFiles\OpenSSH\sftp-server.exe"
            $script:DefaultSftpPath = "sftp-server.exe"
            $script:AlternatePath = "$env:SystemDriveOpenSSH\bin\sftp.exe"
        }
        else {
            $script:PathWithSpaces = "/usr/local/lib/openssh server/sftp-server"
            $script:DefaultSftpPath = "/usr/lib/openssh/sftp-server"
            $script:AlternatePath = "/usr/libexec/sftp-server"
        }
    }

    AfterEach {
        # Clean up test config file after each test
        if (Test-Path $TestConfigPath) {
            Remove-Item -Path $TestConfigPath -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path "$TestConfigPath.bak") {
            Remove-Item -Path "$TestConfigPath.bak" -Force -ErrorAction SilentlyContinue
        }
    }

    Context 'Subsystem keyword' {
        BeforeEach {
            # Create test config with existing subsystems
            $initialContent = @"
Port 22
Subsystem sftp $script:DefaultSftpPath
Subsystem test2 /path/to/test2
PasswordAuthentication yes
"@
            Set-Content -Path $TestConfigPath -Value $initialContent
        }

        It 'Should add a new subsystem that does not already exist' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _exist = $true
                subsystem = @{
                    name = "newsubsystem"
                    value = "/path/to/newsubsystem"
                }
            } | ConvertTo-Json

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify file contains the new subsystem
            $subsystems = Get-Content $TestConfigPath | Where-Object { $_ -match '^\s*subsystem\s+' }
            $subsystems.Count | Should -Be 3
            $subsystems | Should -Contain "subsystem newsubsystem /path/to/newsubsystem"
            # Verify existing subsystems are preserved
            $subsystems | Should -Contain "subsystem sftp $script:DefaultSftpPath"
            $subsystems | Should -Contain "Subsystem test2 /path/to/test2"
        }

        It 'Should treat subsystem names as case-sensitive (SFTP is different from sftp)' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _exist = $true
                subsystem = @{
                    name = "SFTP"  # Uppercase - should be treated as different from lowercase sftp
                    value = $script:AlternatePath
                }
            } | ConvertTo-Json

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify SFTP was added as a new entry (not updating existing sftp) using get
            $getInput = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
            } | ConvertTo-Json
            $result = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result.subsystem.Count | Should -Be 3  # Original sftp + test2 + new SFTP

            # Verify both sftp and SFTP exist as separate entries
            $sftpEntry = $result.subsystem | Where-Object { $_.name -ceq 'sftp' }
            $sftpEntry | Should -Not -BeNullOrEmpty
            $sftpEntry.value | Should -Be $script:DefaultSftpPath

            $SFTPEntry = $result.subsystem | Where-Object { $_.name -ceq 'SFTP' }
            $SFTPEntry | Should -Not -BeNullOrEmpty
            $SFTPEntry.value | Should -Be $script:AlternatePath

            $test2Entry = $result.subsystem | Where-Object { $_.name -ceq 'test2' }
            $test2Entry | Should -Not -BeNullOrEmpty
            $test2Entry.value | Should -Be '/path/to/test2'
        }

        It 'Should remove a subsystem when _exist is false' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _exist = $false
                subsystem = @{
                    name = "sftp"
                }
            } | ConvertTo-Json

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify subsystem was removed
            $subsystems = Get-Content $TestConfigPath | Where-Object { $_ -match '^\s*subsystem\s+' }
            $subsystems.Count | Should -Be 1
            $subsystems | Should -Not -Match 'sftp'
            # Verify other subsystem is still present
            $subsystems | Should -Contain "Subsystem test2 /path/to/test2"
        }

        It 'Should preserve case when adding subsystem with mixed case name' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _exist = $true
                subsystem = @{
                    name = "MyCustomSubsystem"
                    value = "/path/to/custom"
                }
            } | ConvertTo-Json

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify exact case is preserved in file
            $content = Get-Content $TestConfigPath -Raw
            $content | Should -Match "subsystem MyCustomSubsystem /path/to/custom"
        }

        It 'Should handle paths with spaces correctly' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _exist = $true
                subsystem = @{
                    name = "spacepath"
                    value = $script:PathWithSpaces
                }
            } | ConvertTo-Json

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify subsystem with space in path is present
            $subsystems = Get-Content $TestConfigPath | Where-Object { $_ -match '^\s*subsystem\s+' }
            $spacePathLine = $subsystems | Where-Object { $_ -match 'spacepath' }
            $spacePathLine | Should -Not -BeNullOrEmpty
            $spacePathLine | Should -Match ([regex]::Escape($script:PathWithSpaces))
        }

        It 'Should fail when subsystem name is missing' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _exist = $true
                subsystem = @{
                    value = "/path/to/something"
                }
            } | ConvertTo-Json

            $stderrFile = Join-Path $TestDrive "stderr_missing_name.txt"
            sshdconfig set --input $inputConfig -s sshd-config-repeat 2>$stderrFile
            $LASTEXITCODE | Should -Not -Be 0

            Remove-Item -Path $stderrFile -Force -ErrorAction SilentlyContinue
        }

        It 'Should fail with invalid JSON structure' {
            $invalidJson = "{ invalid json }"

            $stderrFile = Join-Path $TestDrive "stderr_invalid_json.txt"
            sshdconfig set --input $invalidJson -s sshd-config-repeat 2>$stderrFile
            $LASTEXITCODE | Should -Not -Be 0

            Remove-Item -Path $stderrFile -Force -ErrorAction SilentlyContinue
        }
    }
}
