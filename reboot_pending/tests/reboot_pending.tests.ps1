# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
    }
}

Describe 'reboot_pending resource tests' {
    It 'should get reboot_pending' -Skip:(!$IsWindows) {
        $out = dsc resource get -r Microsoft.Windows/RebootPending | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.rebootPending | Should -Not -BeNullOrEmpty
    }

    It 'reboot_pending works in a config' -Skip:(!$IsWindows) {
        $ConfigPath = Resolve-Path "$PSScriptRoot/reboot_pending.dsc.yaml"
        $out = dsc config get --file $ConfigPath | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.result.actualState.rebootPending | Should -Not -BeNullOrEmpty
    }

    It 'reboot_pending should have a reason' -Skip:(!$IsWindows -or !$isElevated) {
        $keyPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update"
        $keyName = "RebootRequired"
        try {
            if (-not (Get-ItemProperty "$keyPath\$keyName" -ErrorAction SilentlyContinue)) {
                New-ItemProperty -Path $keyPath -Name $keyName -Value 1 -PropertyType DWord -Force | Out-Null
            }

            $out = dsc resource get -r Microsoft.Windows/RebootPending | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.actualState.reason.count | Should -BeGreaterThan 0 -Because ($out | Out-String)
        } finally {
            Remove-ItemProperty -Path $keyPath -Name $keyName -ErrorAction Ignore
        }
    }
}
