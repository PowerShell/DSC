# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
        $sshdExists = ($null -ne (Get-Command sshd -CommandType Application -ErrorAction Ignore))
        $skipTest = !$isElevated -or !$sshdExists
    }
}

Describe 'SSHDConfig resource tests' -Skip:(!$IsWindows -or $skipTest) {
    BeforeAll {
        $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  properties:
'@
        # set a non-default value in a temporary sshd_config file
        "LogLevel Debug3" | Set-Content -Path $TestDrive/test_sshd_config
    }

    It 'Export works' {
        $out = dsc config export -i "$yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 1
        $out.resources[0].properties | Should -Not -BeNullOrEmpty
        $out.resources[0].properties.port[0] | Should -Be 22
    }

    It 'Get works'{
        $out = dsc config get -i "$yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results.metadata.includeDefaults | Should -Be $true
        $out.results.result.actualState | Should -Not -BeNullOrEmpty
        $out.results.result.actualState.port | Should -Be 22
        $out.results.result.actualState.passwordAuthentication | Should -Be 'yes'
    }

    It 'Get with a specific setting works' {
        $get_yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
    Microsoft.DSC:
        securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  properties:
    passwordauthentication: 'no'
'@
        $out = dsc config get -i "$get_yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        ($out.results.result.actualState.psobject.properties | measure-object).count | Should -Be 1
        $out.results.result.actualState.passwordauthentication | Should -Be 'yes'
    }

    It 'Get with defaults excluded works' {
        $filepath = Join-Path $TestDrive 'test_sshd_config'
        $get_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  properties:
    _metadata:
        includeDefaults: false
        filepath: $filepath
"@
        $out = dsc config get -i "$get_yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results.metadata.includeDefaults | Should -Be $false
        $out.results.result.actualState.count | Should -Be 1
        $out.results.result.actualState.port | Should -Not -Be 22
        $out.results.result.actualState.loglevel | Should -Be 'debug3'
    }
}
