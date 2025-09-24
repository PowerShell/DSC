# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for security features' {
    It 'Unsigned config file gives warning' -Skip:(!$IsWindows) {
        $null = dsc config get -f $PSScriptRoot/../examples/osinfo_parameters.dsc.yaml 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0
        (Get-Content $TestDrive/error.log -Raw) | Should -Match "WARN Authenticode: File '.*?\\osinfo_parameters.dsc.yaml' is not signed.*?"
    }

    It 'Unsigned resource manifest gives warning' -Skip:(!$IsWindows) {
        $null = dsc resource get -r Microsoft/OSInfo 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0
        (Get-Content $TestDrive/error.log -Raw) | Should -Match "WARN Authenticode: File '.*?\\osinfo.dsc.resource.json' is not signed.*?"
    }

    It 'Unsigned resource executable gives warning' -Skip:(!$IsWindows) {
        $null = dsc resource get -r Microsoft/OSInfo 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0
        (Get-Content $TestDrive/error.log -Raw) | Should -Match "WARN Authenticode: File '.*?\\osinfo.exe' is not signed.*?"
    }
}
