# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
        $sshdExists = ($null -ne (Get-Command sshd -CommandType Application -ErrorAction Ignore))
        $skipTest = !$isElevated -or !$sshdExists
    }
}

Describe 'sshd_config Set Tests' -Skip:(!$IsWindows -or $skipTest) {
    BeforeAll {
        # Create a temporary test directory for sshd_config files
        $TestDir = Join-Path $TestDrive "sshd_test"
        New-Item -Path $TestDir -ItemType Directory -Force | Out-Null
        $TestConfigPath = Join-Path $TestDir "sshd_config"
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

    Context 'Set with valid keyword and value' {
        It 'Should set a valid keyword with valid value' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $true
                Port = "1234"
                passwordauthentication = $false
                allowusers = @("user1", "user2")
                ciphers = @("aes128-ctr", "aes192-ctr", "aes256-ctr")
                addressfamily = "inet6"
                authorizedkeysfile = @(".ssh/authorized_keys", ".ssh/authorized_keys2")
            } | ConvertTo-Json

            $output = sshdconfig set --input $inputConfig -s sshd-config 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify file was created
            Test-Path $TestConfigPath | Should -Be $true
            $sshdConfigContents = Get-Content $TestConfigPath
            $sshdConfigContents | Should -Contain "Port 1234"
            $sshdConfigContents | Should -Contain "PasswordAuthentication no"
            $sshdConfigContents | Should -Contain "AllowUsers user1"
            $sshdConfigContents | Should -Contain "AllowUsers user2"
            $sshdConfigContents | Should -Contain "Ciphers aes128-ctr,aes192-ctr,aes256-ctr"
            $sshdConfigContents | Should -Contain "AddressFamily inet6"
            $sshdConfigContents | Should -Contain "AuthorizedKeysFile .ssh/authorized_keys .ssh/authorized_keys2"
        }

        It 'Should set with valid match blocks' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $true
                match = @(
                    @{
                        criteria = @{
                            user = @("alice", "bob")
                        }
                        passwordauthentication = $true
                    },
                    @{
                        criteria = @{
                            group = @("administrators")
                        }
                        permitrootlogin = $false
                    }
                )
            } | ConvertTo-Json -Depth 10

            $output = sshdconfig set --input $inputConfig -s sshd-config 2>$null
            $LASTEXITCODE | Should -Be 0
            Test-Path $TestConfigPath | Should -Be $true
            $sshdConfigContents = Get-Content $TestConfigPath -Raw
            $sshdConfigContents | Should -Match "match user alice,bob"
            $sshdConfigContents | Should -Match "passwordauthentication yes"
            $sshdConfigContents | Should -Match "match group administrators"
            $sshdConfigContents | Should -Match "permitrootlogin no"
        }

        It 'Should create backup when file exists and is not managed by DSC' {
            # Create a non-DSC managed file
            "Port 22`nPermitRootLogin yes" | Set-Content $TestConfigPath

            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $true
                Port = "5555"
            } | ConvertTo-Json

            sshdconfig set --input $inputConfig -s sshd-config 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify backup was created
            $filepath = $TestConfigPath + "_backup"
            Test-Path $filepath | Should -Be $true

            # Verify backup content
            $backupContent = Get-Content $filepath -Raw
            $backupContent | Should -Match "Port 22"
            $backupContent | Should -Match "PermitRootLogin yes"

            # Verify new content using get
            $getInput = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
            } | ConvertTo-Json
            $result = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json
            $result.Port | Should -Be "5555"
        }

        It 'Should not create backup when file is already managed by DSC' {
            # Create a DSC-managed file
            $initialConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $true
                Port = "6789"
            } | ConvertTo-Json

            sshdconfig set --input $initialConfig -s sshd-config 2>$null

            # Update the file
            $newConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $true
                Port = "7777"
            } | ConvertTo-Json

            sshdconfig set --input $newConfig -s sshd-config 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify no backup was created
            Test-Path "$TestConfigPath.bak" | Should -Be $false

            # Verify content using get
            $getInput = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
            } | ConvertTo-Json
            $result = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json
            $result.Port | Should -Be "7777"
        }
    }

    Context 'Set with invalid configuration' {
        It 'Should fail with clobber set to false' {
            $inputConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $false
                Port = "8888"
            } | ConvertTo-Json

            $logFile = Join-Path $TestDrive "clobber_error.log"
            sshdconfig set --input $inputConfig -s sshd-config 2>$logFile
            $LASTEXITCODE | Should -Not -Be 0

            # Read log file and check for error message
            $logContent = Get-Content $logFile -Raw
            $logContent | Should -Match "clobber=false is not yet supported"
        }

        It 'Should fail with invalid keyword and not modify file' {
            # Create initial file with valid config
            $validConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $true
                Port = "9999"
            } | ConvertTo-Json

            sshdconfig set --input $validConfig -s sshd-config 2>$null
            $LASTEXITCODE | Should -Be 0

            # Get original content
            $getInput = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
            } | ConvertTo-Json
            $originalResult = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json

            # Try to set with invalid keyword
            $invalidConfig = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $true
                FakeKeyword = "1234"
            } | ConvertTo-Json

            $output = sshdconfig set --input $invalidConfig -s sshd-config 2>&1
            $LASTEXITCODE | Should -Not -Be 0

            # Verify file content hasn't changed using get
            $currentResult = sshdconfig get --input $getInput -s sshd-config 2>$null | ConvertFrom-Json
            $currentResult.Port | Should -Be "9999"
            $currentResult.Port | Should -Be $originalResult.Port
        }
    }

    Context 'Set with _clobber=false' {
        BeforeEach {
            $initialContent = @"
    Port 2222
    AddressFamily inet
    MaxAuthTries 5
    PermitRootLogin yes
    PasswordAuthentication no
"@
            Set-Content -Path $TestConfigPath -Value $initialContent
        }

        It '<Title>' -TestCases @(
            @{
                Title = 'Should preserve unchanged regular keyword when value is the same'
                InputConfig = @{ Port = "2222" }
                ExpectedContains = @("Port 2222", "AddressFamily inet", "MaxAuthTries 5", "PermitRootLogin yes", "PasswordAuthentication no")
                ExpectedNotContains = @()
                VerifyOrder = @()
            },
            @{
                Title = 'Should overwrite regular keyword when value is different'
                InputConfig = @{ MaxAuthTries = "3" }
                ExpectedContains = @("Port 2222", "AddressFamily inet", "MaxAuthTries 3", "PermitRootLogin yes", "PasswordAuthentication no")
                ExpectedNotContains = @("MaxAuthTries 5")
                VerifyOrder = @()
            },
            @{
                Title = 'Should add regular keyword when it does not exist'
                InputConfig = @{ LoginGraceTime = "60" }
                ExpectedContains = @("Port 2222", "AddressFamily inet", "MaxAuthTries 5", "PermitRootLogin yes", "PasswordAuthentication no", "LoginGraceTime 60")
                ExpectedNotContains = @()
                VerifyOrder = @()
            },
            @{
                Title = 'Should preserve unchanged boolean keyword when value is the same'
                InputConfig = @{ PasswordAuthentication = $false }
                ExpectedContains = @("Port 2222", "AddressFamily inet", "MaxAuthTries 5", "PermitRootLogin yes", "PasswordAuthentication no")
                ExpectedNotContains = @()
                VerifyOrder = @()
            },
            @{
                Title = 'Should overwrite boolean keyword when value is different'
                InputConfig = @{ PasswordAuthentication = $true }
                ExpectedContains = @("Port 2222", "AddressFamily inet", "MaxAuthTries 5", "PermitRootLogin yes", "PasswordAuthentication yes")
                ExpectedNotContains = @("PasswordAuthentication no")
                VerifyOrder = @()
            },
            @{
                Title = 'Should add boolean keyword when it does not exist'
                InputConfig = @{ PubkeyAuthentication = $true }
                ExpectedContains = @("Port 2222", "AddressFamily inet", "MaxAuthTries 5", "PermitRootLogin yes", "PasswordAuthentication no", "PubkeyAuthentication yes")
                ExpectedNotContains = @()
                VerifyOrder = @()
            },
            @{
                Title = 'Should handle multiple keyword changes and preserve order'
                InputConfig = @{
                    PasswordAuthentication = $false
                    PermitRootLogin = $false
                    LoginGraceTime = "60"
                }
                ExpectedContains = @("Port 2222", "AddressFamily inet", "MaxAuthTries 5", "PermitRootLogin no", "PasswordAuthentication no", "LoginGraceTime 60")
                ExpectedNotContains = @("PermitRootLogin yes")
                VerifyOrder = @(
                    @{ Pattern = "^Port"; Before = "^PasswordAuthentication" },
                    @{ Pattern = "^PasswordAuthentication"; Before = "^PermitRootLogin" },
                    @{ Pattern = "^PermitRootLogin"; Before = "^AddressFamily" },
                    @{ Pattern = "^AddressFamily"; Before = "^MaxAuthTries" }
                )
            }
        ) {
            param($Title, $InputConfig, $ExpectedContains, $ExpectedNotContains, $VerifyOrder)

            $config = @{
                _metadata = @{
                    filepath = $TestConfigPath
                }
                _clobber = $false
            }
            foreach ($key in $InputConfig.Keys) {
                $config[$key] = $InputConfig[$key]
            }
            $inputJson = $config | ConvertTo-Json

            $output = sshdconfig set --input $inputJson -s sshd-config 2>$null
            $LASTEXITCODE | Should -Be 0
            $sshdConfigContents = Get-Content $TestConfigPath

            foreach ($expected in $ExpectedContains) {
                $sshdConfigContents | Should -Contain $expected
            }

            foreach ($notExpected in $ExpectedNotContains) {
                $sshdConfigContents | Should -Not -Contain $notExpected
            }

            foreach ($orderCheck in $VerifyOrder) {
                $beforeLine = ($sshdConfigContents | Select-String -Pattern $orderCheck.Pattern).LineNumber
                $afterLine = ($sshdConfigContents | Select-String -Pattern $orderCheck.Before).LineNumber
                $beforeLine | Should -BeLessThan $afterLine
            }
        }
    }
}
