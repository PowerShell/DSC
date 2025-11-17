# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
    }
}

Describe 'Default Shell Configuration Tests' -Skip:(!$IsWindows -or !$isElevated) {
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

            $output = sshdconfig get -s windows-global 2>$null
            $LASTEXITCODE | Should -Be 0

            $result = $output | ConvertFrom-Json
            $result.shell | Should -Be $testShell
            $result.cmdOption | Should -BeNullOrEmpty
            $result.escapeArguments | Should -BeNullOrEmpty
        }

        It 'Should get default shell with args when registry value exists' {
            $testShell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            $testShellWithArgs = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            New-ItemProperty -Path $RegistryPath -Name "DefaultShell" -Value $testShellWithArgs
            New-ItemProperty -Path $RegistryPath -Name "DefaultShellCommandOption" -Value "/c"
            New-ItemProperty -Path $RegistryPath -Name "DefaultShellEscapeArguments" -Value 0 -Type DWord

            $output = sshdconfig get -s windows-global 2>$null
            $LASTEXITCODE | Should -Be 0

            $result = $output | ConvertFrom-Json
            $result.shell | Should -Be $testShell
            $result.cmdOption | Should -Be "/c"
            $result.escapeArguments | Should -Be $false
        }

        It 'Should handle empty default shell registry values' -Skip:(!$IsWindows) {
            $output = sshdconfig get -s windows-global 2>$null
            $LASTEXITCODE | Should -Be 0

            $result = $output | ConvertFrom-Json
            $result.shell | Should -BeNullOrEmpty
            $result.cmdOption | Should -BeNullOrEmpty
            $result.escapeArguments | Should -BeNullOrEmpty
        }
    }

    Context 'Set Default Shell' {
        It 'Should set default shell with valid configuration' {
            $testShell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"

            $inputConfig = @{
                shell = $testShell
                cmdOption = "/c"
                escapeArguments = $false
            } | ConvertTo-Json

            sshdconfig set --input $inputConfig 2>$null
            $LASTEXITCODE | Should -Be 0

            $defaultShell = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $defaultShell.DefaultShell | Should -Be $testShell

            $cmdOption = Get-ItemProperty -Path $RegistryPath -Name "DefaultShellCommandOption" -ErrorAction SilentlyContinue
            $cmdOption.DefaultShellCommandOption | Should -Be "/c"

            $escapeArgs = Get-ItemProperty -Path $RegistryPath -Name "DefaultShellEscapeArguments" -ErrorAction SilentlyContinue
            $escapeArgs.DefaultShellEscapeArguments | Should -Be 0
        }

        It 'Should set default shell with minimal configuration' {
            $inputConfig = @{
                shell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            } | ConvertTo-Json

            sshdconfig set --input $inputConfig 2>$null
            $LASTEXITCODE | Should -Be 0

            $defaultShell = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $defaultShell.DefaultShell | Should -Be "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
        }

        It 'Should handle invalid JSON input gracefully' {
            $invalidJson = "{ invalid json }"

            sshdconfig set --input $invalidJson 2>$null
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Should clear default shell when set to empty string' {
            Set-ItemProperty -Path $RegistryPath -Name "DefaultShell" -Value "C:\Windows\System32\cmd.exe"

            $inputConfig = @{ shell = "" } | ConvertTo-Json

            sshdconfig set --input $inputConfig 2>$null
            $LASTEXITCODE | Should -Be 0

            $result = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $result | Should -BeNullOrEmpty
        }
    }

    Context 'Set then get default shell' {
        It 'Should maintain configuration consistency between set and get' {
            $originalConfig = @{
                shell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
                cmdOption = "/c"
                escapeArguments = $true
            }
            $inputJson = $originalConfig | ConvertTo-Json

            sshdconfig set --input $inputJson 2>$null
            $LASTEXITCODE | Should -Be 0

            $getOutput = sshdconfig get -s windows-global 2>$null
            $LASTEXITCODE | Should -Be 0

            $retrievedConfig = $getOutput | ConvertFrom-Json

            $retrievedConfig.shell | Should -Be $originalConfig.shell
            $retrievedConfig.cmdOption | Should -Be $originalConfig.cmdOption
            $retrievedConfig.escapeArguments | Should -Be $originalConfig.escapeArguments
        }
    }

    Context 'Set default shell with null value' {
        It 'Should clear existing default shell when set to null' {
            $testShell = "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe"
            New-ItemProperty -Path $RegistryPath -Name "DefaultShell" -Value $testShell

            $inputConfig = @{ shell = $null } | ConvertTo-Json

            sshdconfig set --input $inputConfig 2>$null
            $LASTEXITCODE | Should -Be 0

            $result = Get-ItemProperty -Path $RegistryPath -Name "DefaultShell" -ErrorAction SilentlyContinue
            $result | Should -BeNullOrEmpty
        }
    }
}

Describe 'Default Shell Configuration Error Handling on Non-Windows Platforms' -Skip:($IsWindows) {
    It 'Should return error for set command' {
        $inputConfig = @{ shell = $null } | ConvertTo-Json

        $out = sshdconfig set --input $inputConfig 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        $result = $out | ConvertFrom-Json
        $found = $false
        foreach ($item in $result) {
            if (($item.level -eq 'ERROR') -and ($item.fields.message -like '*is only applicable to Windows*')) {
                $found = $true
            }
        }
        $found | Should -Be $true
    }

    It 'Should return error for get command' {
        $out = sshdconfig get -s windows-global 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        $result = $out | ConvertFrom-Json
        $found = $false
        foreach ($item in $result) {
            if (($item.level -eq 'ERROR') -and ($item.fields.message -like '*is only applicable to Windows*')) {
                $found = $true
            }
        }
        $found | Should -Be $true
    }
}
