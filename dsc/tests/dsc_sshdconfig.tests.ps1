# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'SSHDConfig resource tests' {
    BeforeAll {
        $sshdExists = ($null -ne (Get-Command sshd -CommandType Application -ErrorAction Ignore))
        $isAdmin = if ($IsWindows) {
            $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
            [System.Security.Principal.WindowsPrincipal]::new($identity).IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            [System.Environment]::UserName -eq 'root'
        }
        $skipTest = -not ($sshdExists -and $isAdmin)
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
    }

    It 'Export works' -Skip:$skipTest {
        $out = dsc config export -i "$yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 1
        $out.resources[0].properties | Should -Not -BeNullOrEmpty
        $out.resources[0].properties.port[0] | Should -Be 22
    }

    It 'Get works' -Skip:$skipTest {
        $out = dsc config get -i "$yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 1
        $out.resources[0].properties | Should -Not -BeNullOrEmpty
        $out.resources[0].properties.port[0] | Should -Be 22
        $out.resources[0].properties.passwordAuthentication[0] | Should -Be 'yes'
    }

    It 'Get with a specific setting works' -Skip:$skipTest {
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
        $out.results.result.actualState.count | Should -Be 1
        $out.results.result.actualState.passwordauthentication | Should -Be 'yes'
    }

    # get with exclude defaults works
    It 'Get with exclude defaults works' -Skip:$skipTest {
        $get_yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
resources:
- name: sshdconfig
  type: Microsoft.OpenSSH.SSHD/sshd_config
  metadata:
    excludeDefaults: true
  properties:
'@
        $out = dsc config get -i "$get_yaml" | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results.result.actualState.count | Should -Be 1
        $out.results.result.actualState.port | Should -Not -Be 22
        $out.results.result.actualState.authorizedkeys | Should -Not -BeNullOrEmpty
    }
}