# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

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

    It 'reboot_pending should have a reason' -Skip:(!$IsWindows) {
        $keyPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update"
        $keyName = "RebootRequired"
        try {
            if (-not (Get-ItemProperty "$keyPath\$keyName" -ErrorAction SilentlyContinue)) {
                New-ItemProperty -Path $keyPath -Name $keyName -Value 1 -PropertyType DWord -Force | Out-Null
            }

            $out | dsc resource get -r Microsoft.Windows/RebootPending | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.actualState.reason.count | Should -BeGreaterThan 0
        } finally {
            Remove-ItemProperty -Path $keyPath -Name $keyName -ErrorAction Ignore
        }
    }
}
