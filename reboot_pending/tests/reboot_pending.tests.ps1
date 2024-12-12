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
}
