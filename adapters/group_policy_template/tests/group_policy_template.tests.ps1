# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Adapter/GroupPolicyTemplate tests' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
    }

    BeforeAll {
        $adapterType = 'Microsoft.Adapter/GroupPolicyTemplate'
        $resourceType = 'GPO.ControlPanel/Arp'
        $keyPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Policies\Uninstall'
        $valueName = 'NoAddPage'

        function Invoke-GroupPolicyGet {
            param([bool]$Enabled)

            $json = @{
                NoAddPage = $Enabled
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType --adapter $adapterType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }
    }

    It 'Lists resources corresponding to installed ADMX templates' {
        $admxFiles = @(Get-ChildItem -LiteralPath (Join-Path $env:SystemRoot 'PolicyDefinitions') -Filter '*.admx')
        $resources = @(dsc resource list --adapter $adapterType | ConvertFrom-Json)

        $LASTEXITCODE | Should -Be 0
        $admxFiles.Count | Should -BeGreaterThan 0
        $resources.Count | Should -BeGreaterThan 0
        $resources | ForEach-Object {
            $_.requireAdapter | Should -BeExactly $adapterType
            $_.path | Should -Exist
        }
    }

    Context 'Current user policy state' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalKeyExists = Test-Path -LiteralPath $keyPath
            $script:originalValueExists = $false
            $script:originalValue = $null
            $script:originalValueKind = $null
            if (Test-Path -LiteralPath $keyPath) {
                $key = Get-Item -LiteralPath $keyPath
                if ($key.GetValueNames() -contains $valueName) {
                    $script:originalValueExists = $true
                    $script:originalValue = $key.GetValue($valueName, $null, 'DoNotExpandEnvironmentNames')
                    $script:originalValueKind = $key.GetValueKind($valueName)
                }
            }
        }

        AfterAll {
            if ($script:originalValueExists) {
                if (!(Test-Path -LiteralPath $keyPath)) {
                    New-Item -Path $keyPath -Force | Out-Null
                }
                $key = Get-Item -LiteralPath $keyPath
                $key.SetValue($valueName, $script:originalValue, $script:originalValueKind)
            }
            else {
                Remove-ItemProperty -LiteralPath $keyPath -Name $valueName -ErrorAction Ignore
                if (!$script:originalKeyExists -and (Test-Path -LiteralPath $keyPath)) {
                    Remove-Item -LiteralPath $keyPath -ErrorAction Ignore
                }
            }
        }

        It 'Sets and gets a policy in current user scope' {
            $resources = @(dsc resource list $resourceType --adapter $adapterType | ConvertFrom-Json)
            if ($resources.Count -eq 0) {
                Set-ItResult -Skipped -Because 'AddRemovePrograms.admx is not installed.'
                return
            }

            $json = @{
                NoAddPage = $true
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType --adapter $adapterType -f - 2>$TestDrive/error.log

            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            ($out | ConvertFrom-Json).afterState.scope | Should -BeExactly 'currentUser'
            ($out | ConvertFrom-Json).afterState.NoAddPage | Should -BeTrue
            (Invoke-GroupPolicyGet -Enabled $true).NoAddPage | Should -BeTrue
        }
    }
}
