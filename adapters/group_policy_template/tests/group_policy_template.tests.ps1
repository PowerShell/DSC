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
        $moduleStateKeyPath = 'HKCU:\Software\Policies\Microsoft\Windows\PowerShell\ModuleLogging'
        $moduleListKeyPath = Join-Path $moduleStateKeyPath 'ModuleNames'
        $moduleStateValueName = 'EnableModuleLogging'
        $testModuleName = "DscGroupPolicyTemplateTest_$PID"

        function Invoke-GroupPolicyGet {
            param([string]$State)

            $json = @{
                NoAddPage = $State
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
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

    It 'Gets all current user values without input' {
        $resources = @(dsc resource list 'GPO.WindowsComponents/PowerShell' --adapter $adapterType | ConvertFrom-Json)
        if ($resources.Count -eq 0) {
            Set-ItResult -Skipped -Because 'PowerShellExecutionPolicy.admx is not installed.'
            return
        }

        $out = dsc resource get -r 'GPO.WindowsComponents/PowerShell' 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actualState = ($out | ConvertFrom-Json).actualState
        $actualState.scope | Should -BeExactly 'currentUser'
        $actualState.psobject.Properties.Name | Should -Contain 'ModuleLogging'
        $actualState.psobject.Properties.Name | Should -Not -Contain 'EnableModuleLogging'
        $actualState.psobject.Properties.Name | Should -Not -Contain 'EnabledModuleLogging'
        $actualState.ModuleLogging.Listbox_ModuleNames.GetType().IsArray | Should -BeTrue
        foreach ($property in $actualState.psobject.Properties | Where-Object Name -NE 'scope') {
            if ($property.Value -is [string]) {
                $property.Value | Should -BeIn 'enabled', 'disabled', 'notConfigured'
            }
            else {
                $property.Value.state | Should -BeIn 'Enabled', 'Disabled', 'NotConfigured'
            }
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
                NoAddPage = 'enabled'
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log

            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            ($out | ConvertFrom-Json).afterState.scope | Should -BeExactly 'currentUser'
            ($out | ConvertFrom-Json).afterState.NoAddPage | Should -BeExactly 'enabled'
            (Invoke-GroupPolicyGet -State 'enabled').NoAddPage | Should -BeExactly 'enabled'

            $out = dsc resource get -r $resourceType 2>$TestDrive/error.log

            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $actualState = ($out | ConvertFrom-Json).actualState
            $actualState.scope | Should -BeExactly 'currentUser'
            $actualState.NoAddPage | Should -BeExactly 'enabled'

            foreach ($state in 'disabled', 'notConfigured') {
                $json = @{
                    NoAddPage = $state
                } | ConvertTo-Json -Compress
                $out = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log

                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
                ($out | ConvertFrom-Json).afterState.NoAddPage | Should -BeExactly $state
                (Invoke-GroupPolicyGet -State $state).NoAddPage | Should -BeExactly $state
            }
        }

        It 'Gets all configured values for a policy list without input' {
            $resources = @(dsc resource list 'GPO.WindowsComponents/PowerShell' --adapter $adapterType | ConvertFrom-Json)
            if ($resources.Count -eq 0) {
                Set-ItResult -Skipped -Because 'PowerShellExecutionPolicy.admx is not installed.'
                return
            }

            $stateKeyExisted = Test-Path -LiteralPath $moduleStateKeyPath
            $listKeyExisted = Test-Path -LiteralPath $moduleListKeyPath
            $stateValueExisted = $false
            $stateValue = $null
            $stateValueKind = $null
            $listValueExisted = $false
            $listValue = $null
            $listValueKind = $null

            if ($stateKeyExisted) {
                $stateKey = Get-Item -LiteralPath $moduleStateKeyPath
                if ($stateKey.GetValueNames() -contains $moduleStateValueName) {
                    $stateValueExisted = $true
                    $stateValue = $stateKey.GetValue($moduleStateValueName, $null, 'DoNotExpandEnvironmentNames')
                    $stateValueKind = $stateKey.GetValueKind($moduleStateValueName)
                }
            }
            if ($listKeyExisted) {
                $listKey = Get-Item -LiteralPath $moduleListKeyPath
                if ($listKey.GetValueNames() -contains $testModuleName) {
                    $listValueExisted = $true
                    $listValue = $listKey.GetValue($testModuleName, $null, 'DoNotExpandEnvironmentNames')
                    $listValueKind = $listKey.GetValueKind($testModuleName)
                }
            }

            try {
                $json = @{
                    ModuleLogging = @{
                        state = 'Enabled'
                        Listbox_ModuleNames = @($testModuleName)
                    }
                } | ConvertTo-Json -Depth 5 -Compress
                $json | dsc resource set -r 'GPO.WindowsComponents/PowerShell' -f - 2>$TestDrive/error.log | Out-Null
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

                $out = dsc resource get -r 'GPO.WindowsComponents/PowerShell' 2>$TestDrive/error.log

                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
                $actualState = ($out | ConvertFrom-Json).actualState
                $actualState.ModuleLogging.Listbox_ModuleNames | Should -Contain $testModuleName
                $actualState.ModuleLogging.Listbox_ModuleNames | ForEach-Object {
                    $_ | Should -BeOfType [string]
                }
            }
            finally {
                if ($listValueExisted) {
                    (Get-Item -LiteralPath $moduleListKeyPath).SetValue(
                        $testModuleName,
                        $listValue,
                        $listValueKind)
                }
                else {
                    Remove-ItemProperty -LiteralPath $moduleListKeyPath -Name $testModuleName -ErrorAction Ignore
                }
                if ($stateValueExisted) {
                    (Get-Item -LiteralPath $moduleStateKeyPath).SetValue(
                        $moduleStateValueName,
                        $stateValue,
                        $stateValueKind)
                }
                else {
                    Remove-ItemProperty -LiteralPath $moduleStateKeyPath -Name $moduleStateValueName -ErrorAction Ignore
                }
                if (!$listKeyExisted -and (Test-Path -LiteralPath $moduleListKeyPath)) {
                    $listKey = Get-Item -LiteralPath $moduleListKeyPath
                    if ($listKey.ValueCount -eq 0 -and $listKey.SubKeyCount -eq 0) {
                        Remove-Item -LiteralPath $moduleListKeyPath
                    }
                }
                if (!$stateKeyExisted -and (Test-Path -LiteralPath $moduleStateKeyPath)) {
                    $stateKey = Get-Item -LiteralPath $moduleStateKeyPath
                    if ($stateKey.ValueCount -eq 0 -and $stateKey.SubKeyCount -eq 0) {
                        Remove-Item -LiteralPath $moduleStateKeyPath
                    }
                }
            }
        }
    }
}
