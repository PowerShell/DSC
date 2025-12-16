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


Describe 'sshd_config Get and Export Tests' -Skip:($skipTest) {
    BeforeAll {
        $TestConfigPath = Join-Path $TestDrive 'test_sshd_config'
        "LogLevel Debug3`nPasswordAuthentication no" | Set-Content -Path $TestConfigPath
        $configWithMatch = @"
Port 2222
PasswordAuthentication no

Match User testuser
    PasswordAuthentication yes
    AllowTcpForwarding no

Match Address 192.168.1.0/24
    X11Forwarding yes
    MaxAuthTries 3
"@
        $TestConfigPathWithMatch = Join-Path $TestDrive 'test_sshd_config_match'
        $configWithMatch | Set-Content -Path $TestConfigPathWithMatch
        $configWithInclude = @"
Port 3333
Include /etc/ssh/sshd_config.d/*.conf
PasswordAuthentication no
"@
        $TestConfigPathWithInclude = Join-Path $TestDrive 'test_sshd_config_include'
        $configWithInclude | Set-Content -Path $TestConfigPathWithInclude
    }

    AfterAll {
        if (Test-Path $TestConfigPath) {
            Remove-Item -Path $TestConfigPath -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path $TestConfigPathWithMatch) {
            Remove-Item -Path $TestConfigPathWithMatch -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path $TestConfigPathWithInclude) {
            Remove-Item -Path $TestConfigPathWithInclude -Force -ErrorAction SilentlyContinue
        }
    }

    It '<Command> command <Description>' -TestCases @(
        @{
            Command = 'get'
            Description = 'ignores input filtering and returns all properties'
        }
        @{
            Command = 'export'
            Description = 'uses input filtering and returns only specified properties'
        }
    ) {
        param($Command, $Description)

        $inputData = @{
            _metadata = @{
                filepath = $TestConfigPath
            }
            passwordAuthentication = $false
        } | ConvertTo-Json

        if ($Command -eq 'get') {
            $result = sshdconfig $Command --input $inputData -s sshd-config 2>$null | ConvertFrom-Json
        }
        else {
            $result = sshdconfig $Command --input $inputData 2>$null | ConvertFrom-Json
        }

        if ($command -eq 'get') {
            # Get should return all properties regardless of input
            $result.LogLevel | Should -Be "Debug3"
            $result.PasswordAuthentication | Should -Be $false
        }
        else {
            # Export should return only specified properties
            $result.PasswordAuthentication | Should -Be $false
            $result.PSObject.Properties.Name | Should -Not -Contain "loglevel"
        }
    }

    It '<Command> command returns config with match blocks' -TestCases @(
        @{ Command = 'get' }
        @{ Command = 'export' }
    ) {
        param($Command)

        $inputData = @{
            _metadata = @{
                filepath = $TestConfigPathWithMatch
            }
        } | ConvertTo-Json

        if ($Command -eq 'get') {
            $result = sshdconfig $Command --input $inputData -s sshd-config 2>$null | ConvertFrom-Json
        }
        else {
            $result = sshdconfig $Command --input $inputData 2>$null | ConvertFrom-Json
        }
        $result.Port | Should -Be "2222"
        $result.PasswordAuthentication | Should -Be $false
        $result.Match | Should -Not -BeNullOrEmpty
        $result.Match.Count | Should -Be 2
        $result.Match[0].Criteria.User | Should -Be "testuser"
        $result.Match[0].PasswordAuthentication | Should -Be $true
        $result.Match[0].AllowTcpForwarding | Should -Be $false
        $result.Match[1].Criteria.Address | Should -Be "192.168.1.0/24"
        $result.Match[1].X11Forwarding | Should -Be $true
        $result.Match[1].MaxAuthTries | Should -Be "3"
    }

    It '<Command> command displays warning when Include directive is present' -TestCases @(
        @{ Command = 'get' }
        @{ Command = 'export' }
    ) {
        param($Command)

        $inputData = @{
            _metadata = @{
                filepath = $TestConfigPathWithInclude
            }
        } | ConvertTo-Json

        $stderrFile = Join-Path $TestDrive "stderr_$Command.txt"
        if ($Command -eq 'get') {
            $result = sshdconfig $Command --input $inputData -s sshd-config 2>$stderrFile | ConvertFrom-Json
        }
        else {
            $result = sshdconfig $Command --input $inputData 2>$stderrFile | ConvertFrom-Json
        }

        $stderr = Get-Content -Path $stderrFile -Raw -ErrorAction SilentlyContinue
        $stderr | Should -BeLike "*WARN*Include directive found in sshd_config*"
        Remove-Item -Path $stderrFile -Force -ErrorAction SilentlyContinue
    }
}
