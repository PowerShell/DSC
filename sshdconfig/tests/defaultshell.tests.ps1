# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Default Shell Configuration Tests' -Skip:(!$IsWindows) {
    BeforeAll {
        # Store original registry values to restore later
        $OriginalValues = @{}
        $RegistryPath = "HKLM:\SOFTWARE\OpenSSH"
        $ValueNames = @("DefaultShell", "DefaultShellCommandOption", "DefaultShellEscapeArguments")
        $CreatedOpenSSHKey = $false

        # Create OpenSSH registry key if it doesn't exist
        if (-not (Test-Path $RegistryPath)) {
            $CreatedOpenSSHKey = $true
            New-Item -Path $RegistryPath -Force | Out-Null
        }
        else {
            # Store existing values
            foreach ($valueName in $ValueNames) {
                try {
                    $value = Get-ItemProperty -Path $RegistryPath -Name $valueName -ErrorAction SilentlyContinue
                    if ($value) {
                        $OriginalValues[$valueName] = $value.$valueName
                        Remove-ItemProperty -Path $RegistryPath -Name $valueName -ErrorAction SilentlyContinue
                    }
                }
                catch {
                    # Value doesn't currently exist, nothing to store
                }
            }
        }
    }

    AfterAll {
        # Restore original registry values
        if ($CreatedOpenSSHKey) {
            # Remove the OpenSSH key if it was created for the tests
            Remove-Item -Path $RegistryPath -Force -ErrorAction SilentlyContinue
        } else {
            foreach ($valueName in $ValueNames) {
                if ($OriginalValues.ContainsKey($valueName)) {
                    New-ItemProperty -Path $RegistryPath -Name $valueName -Value $OriginalValues[$valueName]
                }
            }
        }
    }

    AfterEach {
        # Clean up any properties set during the tests
        foreach ($valueName in $ValueNames) {
            try {
                Remove-ItemProperty -Path $RegistryPath -Name $valueName -ErrorAction SilentlyContinue
            }
            catch {
                # Ignore if value doesn't exist
            }
        }
    }

    Context 'Get Default Shell' {
        It 'Should get default shell without args when registry value exists' {
            $testShell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            New-ItemProperty -Path $RegistryPath -Name "DefaultShell" -Value $testShell

            $output = sshdconfig get
            $LASTEXITCODE | Should -Be 0

            $result = $output | ConvertFrom-Json
            $result.shell | Should -Be $testShell
            $result.cmd_option | Should -BeNullOrEmpty
            $result.escape_arguments | Should -BeNullOrEmpty
            $result.shell_arguments | Should -BeNullOrEmpty
        }

        It 'Should get default shell with args when registry value exists' {
            $testShell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            $testShellWithArgs = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -NonInteractive"
            New-ItemProperty -Path $RegistryPath -Name "DefaultShell" -Value $testShellWithArgs
            New-ItemProperty -Path $RegistryPath -Name "DefaultShellCommandOption" -Value "/c"
            New-ItemProperty -Path $RegistryPath -Name "DefaultShellEscapeArguments" -Value 0 -Type DWord

            $output = sshdconfig get
            $LASTEXITCODE | Should -Be 0

            $result = $output | ConvertFrom-Json
            $result.shell | Should -Be $testShell
            $result.cmd_option | Should -Be "/c"
            $result.escape_arguments | Should -Be $false
            $result.shell_arguments | Should -Be @("-NoProfile", "-NonInteractive")
        }

        It 'Should handle empty default shell registry values' -Skip:(!$IsWindows) {
            $output = sshdconfig get
            $LASTEXITCODE | Should -Be 0

            $result = $output | ConvertFrom-Json
            $result.shell | Should -BeNullOrEmpty
            $result.cmd_option | Should -BeNullOrEmpty
            $result.escape_arguments | Should -BeNullOrEmpty
            $result.shell_arguments | Should -BeNullOrEmpty
        }
    }

    Context 'Set Default Shell' {
        It 'Should set default shell with valid configuration' {
            $testShell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            $testShellWithArgs = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -NonInteractive"

            $inputConfig = @{
                shell = $testShell
                cmd_option = "/c"
                escape_arguments = $false
                shell_arguments = @("-NoProfile", "-NonInteractive")
            } | ConvertTo-Json

            sshdconfig set --input $inputConfig
            $LASTEXITCODE | Should -Be 0

            $defaultShell = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $defaultShell.DefaultShell | Should -Be $testShellWithArgs

            $cmdOption = Get-ItemProperty -Path $RegistryPath -Name "DefaultShellCommandOption" -ErrorAction SilentlyContinue
            $cmdOption.DefaultShellCommandOption | Should -Be "/c"

            $escapeArgs = Get-ItemProperty -Path $RegistryPath -Name "DefaultShellEscapeArguments" -ErrorAction SilentlyContinue
            $escapeArgs.DefaultShellEscapeArguments | Should -Be 0
        }

        It 'Should set default shell with minimal configuration' {
            $inputConfig = @{
                shell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            } | ConvertTo-Json

            sshdconfig set --input $inputConfig
            $LASTEXITCODE | Should -Be 0

            $defaultShell = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $defaultShell.DefaultShell | Should -Be "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
        }

        It 'Should handle invalid JSON input gracefully' {
            $invalidJson = "{ invalid json }"

            sshdconfig set --input $invalidJson
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Should clear default shell when set to null' {
            Set-ItemProperty -Path $RegistryPath -Name "DefaultShell" -Value "C:\Windows\System32\cmd.exe"

            $inputConfig = @{ shell = $null } | ConvertTo-Json

            sshdconfig set --input $inputConfig
            $LASTEXITCODE | Should -Be 0

            $result = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $result | Should -BeNullOrEmpty
        }
    }

    Context 'Set then get default shell' {
        It 'Should maintain configuration consistency between set and get' {
            $originalConfig = @{
                shell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
                cmd_option = "/c"
                escape_arguments = $true
                shell_arguments = @("-NoProfile", "-NonInteractive")
            }
            $inputJson = $originalConfig | ConvertTo-Json

            sshdconfig set --input $inputJson
            $LASTEXITCODE | Should -Be 0

            $getOutput = sshdconfig get
            $LASTEXITCODE | Should -Be 0

            $retrievedConfig = $getOutput | ConvertFrom-Json

            $retrievedConfig.shell | Should -Be $originalConfig.shell
            $retrievedConfig.cmd_option | Should -Be $originalConfig.cmd_option
            $retrievedConfig.escape_arguments | Should -Be $originalConfig.escape_arguments
            $retrievedConfig.shell_arguments | Should -Be $originalConfig.shell_arguments
        }
    }

    Context 'Set default shell with null value' {
        It 'Should clear existing default shell when set to null' {
            $testShell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            New-ItemProperty -Path $RegistryPath -Name "DefaultShell" -Value $testShell

            $inputConfig = @{ shell = $null } | ConvertTo-Json

            sshdconfig set --input $inputConfig
            $LASTEXITCODE | Should -Be 0

            $result = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $result | Should -BeNullOrEmpty
        }
    }
}
