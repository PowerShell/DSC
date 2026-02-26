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

Describe 'sshd-config-repeat-list Set Tests' -Skip:($skipTest) {
    BeforeAll {
        # Create a temporary test directory for sshd_config files
        $TestDir = Join-Path $TestDrive "sshd_test"
        New-Item -Path $TestDir -ItemType Directory -Force | Out-Null
        $TestConfigPath = Join-Path $TestDir "sshd_config"

        # Define OS-specific paths with spaces
        if ($IsWindows) {
            $script:PathWithSpaces = "$env:ProgramFiles\OpenSSH\sftp-server.exe"
            $script:DefaultSftpPath = "sftp-server.exe"
            $script:AlternatePath = "$env:SystemDrive\OpenSSH\bin\sftp.exe"
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
Port 1234
subsystem sftp $script:DefaultSftpPath
Subsystem test2 /path/to/test2
PasswordAuthentication yes
"@
            Set-Content -Path $TestConfigPath -Value $initialContent
        }

        It 'Should add multiple new subsystems' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $false
                subsystem = @(
                    @{
                        name = "powershell"
                        value = "/usr/bin/pwsh -sshs -NoLogo"
                    },
                    @{
                        name = "newsub2"
                        value = "/path/to/newsub2"
                    }
                )
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify all subsystems are present (old + new) using get
            $getInput = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
            } | ConvertTo-Json
            $result = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result.port | Should -Be 1234 # Verify non-subsystem lines are preserved
            $result.subsystem.Count | Should -Be 4  # 2 existing + 2 new

            # Verify new subsystems were added
            $newsub1 = $result.subsystem | Where-Object { $_.name -ceq 'powershell' }
            $newsub1 | Should -Not -BeNullOrEmpty
            $newsub1.value | Should -Be '/usr/bin/pwsh -sshs -NoLogo'

            $newsub2 = $result.subsystem | Where-Object { $_.name -ceq 'newsub2' }
            $newsub2 | Should -Not -BeNullOrEmpty
            $newsub2.value | Should -Be '/path/to/newsub2'

            # Verify existing subsystems are preserved
            ($result.subsystem | Where-Object { $_.name -ceq 'sftp' }) | Should -Not -BeNullOrEmpty
            ($result.subsystem | Where-Object { $_.name -ceq 'test2' }) | Should -Not -BeNullOrEmpty
        }

        It 'Should update existing subsystems and add new ones' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $false
                subsystem = @(
                    @{
                        name = "sftp"  # Update existing
                        value = $script:AlternatePath
                    },
                    @{
                        name = "newsub"  # Add new
                        value = "/path/to/newsub"
                    }
                )
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify subsystems
            $subsystems = Get-Content $TestConfigPath | Where-Object { $_ -match '^\s*subsystem\s+' }

            # Should have updated sftp, kept test2, and added newsub
            $subsystems.Count | Should -Be 3

            # Verify sftp was updated
            $sftpLine = $subsystems | Where-Object { $_ -match 'sftp' }
            $sftpLine | Should -Match ([regex]::Escape($script:AlternatePath))

            # Verify new subsystem was added
            $subsystems | Should -Contain "subsystem newsub /path/to/newsub"

            # Verify unlisted subsystems are preserved (_purge: false)
            $subsystems | Should -Contain "Subsystem test2 /path/to/test2"
        }

        It 'Should preserve unlisted subsystems when _purge is false' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $false
                subsystem = @(
                    @{
                        name = "onlythisone"
                        value = "/path/to/this"
                    }
                )
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify all existing subsystems are still present plus the new one using get
            $getInput = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
            } | ConvertTo-Json
            $result = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result.subsystem.Count | Should -Be 3  # 2 existing + 1 new

            # Verify new subsystem
            $newEntry = $result.subsystem | Where-Object { $_.name -ceq 'onlythisone' }
            $newEntry | Should -Not -BeNullOrEmpty
            $newEntry.value | Should -Be '/path/to/this'

            # Verify all existing subsystems preserved
            ($result.subsystem | Where-Object { $_.name -ceq 'sftp' }) | Should -Not -BeNullOrEmpty
            ($result.subsystem | Where-Object { $_.name -ceq 'test2' }) | Should -Not -BeNullOrEmpty
        }

        It 'Should remove unlisted subsystems when _purge is true' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $true
                subsystem = @(
                    @{
                        name = "sftp"
                        value = $script:AlternatePath
                    },
                    @{
                        name = "newsub"
                        value = "/path/to/newsub"
                    }
                )
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify only specified subsystems remain using get
            $getInput = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
            } | ConvertTo-Json
            $result = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0

            $result.subsystem.Count | Should -Be 2

            # Verify specified subsystems are present
            $sftpEntry = $result.subsystem | Where-Object { $_.name -ceq 'sftp' }
            $sftpEntry | Should -Not -BeNullOrEmpty
            $sftpEntry.value | Should -Be $script:AlternatePath

            $newsubEntry = $result.subsystem | Where-Object { $_.name -ceq 'newsub' }
            $newsubEntry | Should -Not -BeNullOrEmpty
            $newsubEntry.value | Should -Be '/path/to/newsub'

            # Verify unlisted subsystems were removed
            ($result.subsystem | Where-Object { $_.name -ceq 'test2' }) | Should -BeNullOrEmpty
            ($result.subsystem | Where-Object { $_.name -ceq 'internal-sftp' }) | Should -BeNullOrEmpty
        }

        It 'Should preserve case in subsystem names' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $false
                subsystem = @(
                    @{
                        name = "MixedCase"
                        value = "/path/to/mixed"
                    },
                    @{
                        name = "UPPERCASE"
                        value = "/path/to/upper"
                    }
                )
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify exact case is preserved in file
            $content = Get-Content $TestConfigPath -Raw
            $content | Should -Match "subsystem MixedCase /path/to/mixed"
            $content | Should -Match "subsystem UPPERCASE /path/to/upper"
        }

        It 'Should handle paths with spaces in list' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $false
                subsystem = @(
                    @{
                        name = "spacepath1"
                        value = $script:PathWithSpaces
                    },
                    @{
                        name = "spacepath2"
                        value = "/another/path with spaces/binary"
                    }
                )
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify subsystems with spaces in paths are present
            $subsystems = Get-Content $TestConfigPath | Where-Object { $_ -match '^\s*subsystem\s+' }
            $spacePath1Line = $subsystems | Where-Object { $_ -match 'spacepath1' }
            $spacePath2Line = $subsystems | Where-Object { $_ -match 'spacepath2' }

            $spacePath1Line | Should -Not -BeNullOrEmpty
            $spacePath2Line | Should -Not -BeNullOrEmpty
            $spacePath1Line | Should -Match ([regex]::Escape($script:PathWithSpaces))
            $spacePath2Line | Should -Match '/another/path with spaces/binary'
        }

        It 'Should fail with missing required name property' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $false
                subsystem = @(
                    @{
                        value = "/path/to/something"
                    }
                )
            } | ConvertTo-Json -Depth 10

            $stderrFile = Join-Path $TestDrive "stderr_missing_property.txt"
            sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$stderrFile
            $LASTEXITCODE | Should -Not -Be 0

            Remove-Item -Path $stderrFile -Force -ErrorAction SilentlyContinue
        }

        It 'Should handle empty subsystem array when _purge is true' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _purge = $true
                subsystem = @()
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config-repeat-list 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify all subsystems were removed
            $subsystems = Get-Content $TestConfigPath | Where-Object { $_ -match '^\s*subsystem\s+' }
            $subsystems.Count | Should -Be 0
        }
    }
}
