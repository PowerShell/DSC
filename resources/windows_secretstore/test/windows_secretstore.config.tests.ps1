# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows SecretStore config tests' -Skip:(!$IsWindows) {
    BeforeAll {
        Set-StrictMode -Version Latest

        function Install-TestModule {
            param(
                [Parameter(Mandatory = $true)]
                [string]$Name
            )

            if (Get-Module -ListAvailable -Name $Name -ErrorAction SilentlyContinue | Select-Object -First 1) {
                return
            }

            if (-not (Get-PackageProvider -Name NuGet -ListAvailable -ErrorAction SilentlyContinue)) {
                Install-PackageProvider -Name NuGet -MinimumVersion '2.8.5.201' -Force -Scope CurrentUser | Out-Null
            }

            Install-Module -Name $Name -Repository PSGallery -Scope CurrentUser -Force -AllowClobber -Confirm:$false -ErrorAction Stop
        }

        foreach ($moduleName in @(
            'Pester',
            'Microsoft.PowerShell.SecretManagement',
            'Microsoft.PowerShell.SecretStore'
        )) {
            Install-TestModule -Name $moduleName
        }

        Import-Module Microsoft.PowerShell.SecretManagement -Force -ErrorAction Stop
        Import-Module Microsoft.PowerShell.SecretStore -Force -ErrorAction Stop

        function Reset-SecretStoreForTest {
            Reset-SecretStore -Authentication None -Interaction None -PasswordTimeout -1 -Force -Confirm:$false -ErrorAction Stop | Out-Null
        }

        # Inline DSC config: none-auth (unattended automation)
        $script:configNone = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Configure SecretStore for unattended automation
    type: Microsoft.PowerShell/WindowsSecretStore
    properties:
      authentication: None
      passwordTimeout: -1
      interaction: None
      scope: CurrentUser
'@

        # Inline DSC config: password-auth with secureString parameter
        $script:configPassword = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  SecretPassword:
    type: secureString
    defaultValue: TestSecretValue
resources:
  - name: Configure SecretStore for unattended automation
    type: Microsoft.PowerShell/WindowsSecretStore
    properties:
      authentication: Password
      passwordTimeout: -1
      interaction: None
      scope: CurrentUser
      password: "[parameters('SecretPassword')]"
'@
    }

    BeforeEach {
        Reset-SecretStoreForTest
    }

    Context 'dsc config set then test (none-auth)' {
        It 'applies the none-auth configuration via dsc config set' {
            $null = dsc config set -i $script:configNone 2>"$TestDrive/set.stderr"
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/set.stderr")
        }

        It 'reports desired state via dsc config test' {
            $null = dsc config set -i $script:configNone 2>"$TestDrive/set.stderr"
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/set.stderr")

            $out = dsc config test -i $script:configNone 2>"$TestDrive/test.stderr" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/test.stderr")

            $result = $out.results[0].result
            $result.inDesiredState | Should -BeTrue
        }

        It 'returns current state via dsc config get' {
            $null = dsc config set -i $script:configNone 2>"$TestDrive/set.stderr"
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/set.stderr")

            $out = dsc config get -i $script:configNone 2>"$TestDrive/get.stderr" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/get.stderr")

            $actualState = $out.results[0].result.actualState
            $actualState.authentication | Should -BeExactly 'None'
            $actualState.interaction   | Should -BeExactly 'None'
            $actualState.passwordTimeout | Should -Be -1
            $actualState.scope         | Should -BeExactly 'CurrentUser'
        }
    }

    Context 'dsc config set then test (password-auth)' {
        It 'applies the password-auth configuration via dsc config set' {
            $null = dsc config set -i $script:configPassword 2>"$TestDrive/set.stderr"
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/set.stderr")
        }

        It 'reports desired state via dsc config test' {
            $null = dsc config set -i $script:configPassword 2>"$TestDrive/set.stderr"
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/set.stderr")

            $out = dsc config test -i $script:configPassword 2>"$TestDrive/test.stderr" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/test.stderr")

            $result = $out.results[0].result
            $result.inDesiredState | Should -BeTrue
        }

        It 'returns current state via dsc config get' {
            $null = dsc config set -i $script:configPassword 2>"$TestDrive/set.stderr"
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/set.stderr")

            $out = dsc config get -i $script:configPassword 2>"$TestDrive/get.stderr" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw "$TestDrive/get.stderr")

            $actualState = $out.results[0].result.actualState
            $actualState.authentication  | Should -BeExactly 'Password'
            $actualState.interaction     | Should -BeExactly 'None'
            $actualState.passwordTimeout | Should -Be -1
            $actualState.scope           | Should -BeExactly 'CurrentUser'
        }
    }
}