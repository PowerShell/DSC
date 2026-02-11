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
        $out = dsc config get -f $PSScriptRoot/../examples/require_admin.yaml 2>$null
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
        else {
            $LASTEXITCODE | Should -Be 2
        }
    }

    It 'Require admin with warning deprecated' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: os
  type: Microsoft/OSInfo
  properties: {}
'@
        $out = dsc config get -i $configYaml 2>$testdrive/error.log
        $errorLog = Get-Content -Path $testdrive/error.log -Raw
        $errorLog | Should -Contain 'Using Microsoft.DSC metadata to specify required security context is deprecated. Please use the ''securityContext'' directive in the configuration document instead.'
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
        else {
            $LASTEXITCODE | Should -Be 2
        }
    }

    It 'Require non-admin' {
        $out = dsc config get -f $PSScriptRoot/../examples/require_nonadmin.yaml 2>$null
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 2
        }
        else {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
    }

    It 'Require admin with conflicting metadata and directive' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
directives:
  securityContext: restricted
resources:
- name: os
  type: Microsoft/OSInfo
  properties: {}
'@
        $out = dsc config get -i $configYaml 2>$testdrive/error.log
        $errorLog = Get-Content -Path $testdrive/error.log -Raw
        $errorLog | Should -Contain 'Conflicting security context specified in configuration document: metadata ''elevated'' and directive ''restricted'''
        $LASTEXITCODE | Should -Be 2
    }

    It 'Require non-admin with warning deprecated' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: non-elevated
resources:
- name: os
  type: Microsoft/OSInfo
  properties: {}
'@
        $out = dsc config get -i $configYaml 2>$testdrive/error.log
        $errorLog = Get-Content -Path $testdrive/error.log -Raw
        $errorLog | Should -Contain 'Using Microsoft.DSC metadata to specify required security context is deprecated. Please use the ''securityContext'' directive in the configuration document instead.'
        if ($isAdmin) {
            $LASTEXITCODE | Should -Be 2
        }
        else {
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
        }
    }
}
