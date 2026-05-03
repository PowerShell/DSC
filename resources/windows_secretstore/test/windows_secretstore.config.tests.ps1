# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows SecretStore config tests' -Skip:(!$IsWindows) {
    BeforeAll {
        Set-StrictMode -Version Latest

        $script:resourceRoot = Split-Path -Parent $PSScriptRoot
        $script:resourceScript = Join-Path $script:resourceRoot 'windows_secretstore.ps1'
        $script:configRoot = 'C:\Users\mmach\OneDrive\Skrypty\DSC\3.0\DSC\config'

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
            'powershell-yaml',
            'Microsoft.PowerShell.SecretManagement',
            'Microsoft.PowerShell.SecretStore'
        )) {
            Install-TestModule -Name $moduleName
        }

        Import-Module powershell-yaml -Force -ErrorAction Stop
        Import-Module Microsoft.PowerShell.SecretManagement -Force -ErrorAction Stop
        Import-Module Microsoft.PowerShell.SecretStore -Force -ErrorAction Stop

        function Read-ConfigFile {
            param(
                [Parameter(Mandatory = $true)]
                [string]$Path
            )

            $yaml = ConvertFrom-Yaml -Yaml (Get-Content -Raw $Path -ErrorAction Stop)
            return $yaml | ConvertTo-Json -Depth 20 | ConvertFrom-Json -AsHashtable
        }

        $script:configs = @{
            none = Read-ConfigFile -Path (Join-Path $script:configRoot 'secret_store_none.yaml')
            secure = Read-ConfigFile -Path (Join-Path $script:configRoot 'secret_store_secure.yaml')
        }

        function Convert-ConfigToDesiredState {
            param(
                [Parameter(Mandatory = $true)]
                [object]$Config
            )

            $properties = [ordered]@{}
            $resourceProperties = $Config['resources'][0]['properties']

            foreach ($propertyName in $resourceProperties.Keys) {
                $properties[$propertyName] = $resourceProperties[$propertyName]
            }

            $parameters = $Config['parameters']
            if ($null -eq $parameters) {
                return $properties
            }

            foreach ($parameterName in $parameters.Keys) {
                $parameter = $parameters[$parameterName]
                $parameterType = $parameter['type']
                $defaultValue = $parameter['defaultValue']

                if ($parameterType -eq 'secureString' -and $properties['password'] -eq "[parameters('$parameterName')]") {
                    $properties['password'] = @{ secureString = [string]$defaultValue }
                }
            }

            return $properties
        }

        function Invoke-SecretStoreOperation {
            param(
                [Parameter(Mandatory = $true)]
                [ValidateSet('Get', 'Set', 'Test')]
                [string]$Operation,

                [Parameter(Mandatory = $true)]
                [hashtable]$DesiredState
            )

            $jsonInput = $DesiredState | ConvertTo-Json -Depth 10 -Compress
            Remove-Variable -Name LASTEXITCODE -Scope Global -ErrorAction SilentlyContinue
            $output = & $script:resourceScript $Operation $jsonInput 2>$testdrive/windows_secretstore.stderr
            $lastExitCode = Get-Variable -Name LASTEXITCODE -Scope Global -ErrorAction SilentlyContinue
            $exitCode = if ($null -ne $lastExitCode) { [int]$lastExitCode.Value } else { 0 }
            $errorText = if (Test-Path $testdrive/windows_secretstore.stderr) {
                Get-Content -Raw $testdrive/windows_secretstore.stderr
            }
            else {
                ''
            }

            [pscustomobject]@{
                ExitCode = $exitCode
                StdOut = $output
                StdErr = $errorText
                State = if ($output) { $output | ConvertFrom-Json -Depth 10 } else { $null }
            }
        }

        function Reset-SecretStoreForTest {
            Reset-SecretStore -Authentication None -Interaction None -PasswordTimeout -1 -Force -Confirm:$false -ErrorAction Stop | Out-Null
        }
    }

    BeforeEach {
        Reset-SecretStoreForTest
    }

    Context 'secret_store_none.yaml' {
        It 'applies the none-auth configuration and reports desired state' {
            $desiredState = Convert-ConfigToDesiredState -Config $script:configs.none

            $setResult = Invoke-SecretStoreOperation -Operation Set -DesiredState $desiredState
            $setResult.ExitCode | Should -Be 0 -Because $setResult.StdErr

            $testResult = Invoke-SecretStoreOperation -Operation Test -DesiredState $desiredState
            $testResult.ExitCode | Should -Be 0 -Because $testResult.StdErr
            $testResult.State.authentication | Should -BeExactly 'None'
            $testResult.State.interaction | Should -BeExactly 'None'
            $testResult.State.passwordTimeout | Should -Be -1
            $testResult.State.scope | Should -BeExactly 'CurrentUser'
            $testResult.State._inDesiredState | Should -BeTrue
        }
    }

    Context 'secret_store_secure.yaml' {
        It 'accepts the secureString parameter shape and reports desired state' {
            $desiredState = Convert-ConfigToDesiredState -Config $script:configs.secure

            $setResult = Invoke-SecretStoreOperation -Operation Set -DesiredState $desiredState
            $setResult.ExitCode | Should -Be 0 -Because $setResult.StdErr

            $testResult = Invoke-SecretStoreOperation -Operation Test -DesiredState $desiredState
            $testResult.ExitCode | Should -Be 0 -Because $testResult.StdErr
            $testResult.State.authentication | Should -BeExactly 'Password'
            $testResult.State.interaction | Should -BeExactly 'None'
            $testResult.State.passwordTimeout | Should -Be -1
            $testResult.State.scope | Should -BeExactly 'CurrentUser'
            $testResult.State._inDesiredState | Should -BeTrue
        }

        It 'returns the configured state when Get is called with the secureString password shape' {
            $desiredState = Convert-ConfigToDesiredState -Config $script:configs.secure

            $setResult = Invoke-SecretStoreOperation -Operation Set -DesiredState $desiredState
            $setResult.ExitCode | Should -Be 0 -Because $setResult.StdErr

            $getResult = Invoke-SecretStoreOperation -Operation Get -DesiredState $desiredState
            $getResult.ExitCode | Should -Be 0 -Because $getResult.StdErr
            $getResult.State.authentication | Should -BeExactly 'Password'
            $getResult.State.interaction | Should -BeExactly 'None'
            $getResult.State.passwordTimeout | Should -Be -1
            $getResult.State.scope | Should -BeExactly 'CurrentUser'
        }
    }
}