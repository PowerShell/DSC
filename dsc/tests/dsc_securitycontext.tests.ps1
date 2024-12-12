# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for configuration security context metadata' {
    BeforeAll {
        $isAdmin = if ($IsWindows) {
            $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
            [System.Security.Principal.WindowsPrincipal]::new($identity).IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            [System.Environment]::UserName -eq 'root'
        }
    }

    It 'Require admin' {
        $out = dsc config get -f $PSScriptRoot/../examples/require_admin.yaml
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
        else {
            $LASTEXITCODE | Should -Be 2
        }
    }

    It 'Require non-admin' {
        $out = dsc config get -f $PSScriptRoot/../examples/require_nonadmin.yaml
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 2
        }
        else {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
    }
}
