# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
    }
}

Describe 'reboot_pending resource tests' -Skip:(!$IsWindows -or !$isElevated) {
    BeforeAll {
        $keyPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update"
        $keyName = "RebootRequired"
        if (-not (Get-ItemProperty "$keyPath\$keyName" -ErrorAction Ignore)) {
            New-ItemProperty -Path $keyPath -Name $keyName -Value 1 -PropertyType DWord -Force | Out-Null
        }
    }

    AfterAll {
        Remove-ItemProperty -Path $keyPath -Name $keyName -ErrorAction Ignore
    }

    It 'should get reboot_pending' {
        $out = dsc resource get -r Microsoft.Windows/RebootPending | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.rebootPending | Should -Not -BeNullOrEmpty
    }

    It 'reboot_pending works in a config' {
        $ConfigPath = Resolve-Path "$PSScriptRoot/reboot_pending.dsc.yaml"
        $out = dsc config get --file $ConfigPath | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.result.actualState.rebootPending | Should -Not -BeNullOrEmpty
    }

    It 'reboot_pending should have a reason' {
        $out = dsc resource get -r Microsoft.Windows/RebootPending | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($out.actualState.rebootPending) {
            $out.actualState.reason.count | Should -BeGreaterThan 0 -Because ($out | ConvertTo-Json -Depth 10 |Out-String)
        } else {
            $out.actualState.reason | Should -BeNullOrEmpty -Because ($out | ConvertTo-Json -Depth 10 |Out-String)
        }
    }
}
