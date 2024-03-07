# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for configuration security context metadata' {
    BeforeAll {
        $isAdmin = if ($IsWindows) {
            [System.Security.Principal.WindowsIdentity]::GetCurrent().Owner.IsWellKnown('Builtin\Administrators')
        }
        else {
            [System.Environment]::UserName -eq 'root'
        }
    }

    It 'Require admin' {
        $out = dsc config get -p $PSScriptRoot/../examples/require_admin.yaml
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
        else {
            $LASTEXITCODE | Should -Be 2
        }
    }

    It 'Require non-admin' {
        $out = dsc config get -p $PSScriptRoot/../examples/require_nonadmin.yaml
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 2
        }
        else {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
    }
}
