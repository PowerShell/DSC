# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry offline hive tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $testHivesSource = Join-Path $PSScriptRoot 'test_hives'
    }

    Context 'Get from offline HKLM hive' {
        BeforeAll {
            $script:hklmHive = Join-Path $TestDrive 'HKLM.hiv'
            Copy-Item (Join-Path $testHivesSource 'HKLM.hiv') -Destination $script:hklmHive
        }

        It 'Can get a registry key from offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $out = registry config get --input $json 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result.keyPath | Should -Be 'HKLM\Software\DSCTest'
        }

        It 'Can get a string value from offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestString'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $out = registry config get --input $json 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result.keyPath | Should -Be 'HKLM\Software\DSCTest'
            $result.valueName | Should -Be 'TestString'
            $result.valueData.String | Should -Be 'TestValue'
        }

        It 'Can get a DWORD value from offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestDword'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $out = registry config get --input $json 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result.keyPath | Should -Be 'HKLM\Software\DSCTest'
            $result.valueName | Should -Be 'TestDword'
            $result.valueData.DWord | Should -Be 42
        }

        It 'Returns _exist false for non-existent key in offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\NonExistent'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $out = registry config get --input $json 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result._exist | Should -Be $false
        }

        It 'Returns _exist false for non-existent value in offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'DoesNotExist'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $out = registry config get --input $json 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result._exist | Should -Be $false
        }
    }

    Context 'Get from offline HKCU hive' {
        BeforeAll {
            $script:hkcuHive = Join-Path $TestDrive 'HKCU.hiv'
            Copy-Item (Join-Path $testHivesSource 'HKCU.hiv') -Destination $script:hkcuHive
        }

        It 'Can get a string value from offline HKCU hive' {
            $json = @{
                keyPath = 'HKCU\Software\DSCUserTest'
                valueName = 'UserString'
                registryFilePath = $script:hkcuHive
            } | ConvertTo-Json -Compress
            $out = registry config get --input $json 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result.keyPath | Should -Be 'HKCU\Software\DSCUserTest'
            $result.valueName | Should -Be 'UserString'
            $result.valueData.String | Should -Be 'UserValue'
        }
    }

    Context 'Set in offline hive' {
        BeforeEach {
            $script:hklmHive = Join-Path $TestDrive 'HKLM_set.hiv'
            Copy-Item (Join-Path $testHivesSource 'HKLM.hiv') -Destination $script:hklmHive
        }

        It 'Can set a new value in offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'NewValue'
                valueData = @{ String = 'Hello' }
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress -Depth 3
            $out = registry config set --input $json 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify the value was written
            $getJson = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'NewValue'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $result = registry config get --input $getJson 2>$null | ConvertFrom-Json
            $result.valueData.String | Should -Be 'Hello'
        }

        It 'Can create a new key in offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\NewKey\SubKey'
                valueName = 'Test'
                valueData = @{ DWord = 99 }
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress -Depth 3
            $null = registry config set --input $json 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify
            $getJson = @{
                keyPath = 'HKLM\Software\NewKey\SubKey'
                valueName = 'Test'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $result = registry config get --input $getJson 2>$null | ConvertFrom-Json
            $result.valueData.DWord | Should -Be 99
        }
    }

    Context 'Delete from offline hive' {
        BeforeEach {
            $script:hklmHive = Join-Path $TestDrive 'HKLM_delete.hiv'
            Copy-Item (Join-Path $testHivesSource 'HKLM.hiv') -Destination $script:hklmHive
        }

        It 'Can delete a value from offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestString'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $null = registry config delete --input $json 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify value is gone
            $result = registry config get --input $json 2>$null | ConvertFrom-Json
            $result._exist | Should -Be $false
        }

        It 'Can delete a key from offline hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $null = registry config delete --input $json 2>$null
            $LASTEXITCODE | Should -Be 0

            # Verify key is gone
            $result = registry config get --input $json 2>$null | ConvertFrom-Json
            $result._exist | Should -Be $false
        }
    }

    Context 'RegistryList with offline hive' {
        BeforeAll {
            $script:hklmHive = Join-Path $TestDrive 'HKLM_list.hiv'
            Copy-Item (Join-Path $testHivesSource 'HKLM.hiv') -Destination $script:hklmHive
        }

        It 'Can get multiple values from offline hive using RegistryList' {
            $listJson = @{
                registryFilePath = $script:hklmHive
                registryEntries = @(
                    @{ keyPath = 'HKLM\Software\DSCTest'; valueName = 'TestString' }
                    @{ keyPath = 'HKLM\Software\DSCTest'; valueName = 'TestDword' }
                    @{ keyPath = 'HKLM\Software\DSCTest'; valueName = 'NonExistent' }
                )
            } | ConvertTo-Json -Compress -Depth 3
            $out = registry config get --list --input $listJson 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result.registryEntries.Count | Should -Be 3
            $result.registryEntries[0].valueData.String | Should -Be 'TestValue'
            $result.registryEntries[1].valueData.DWord | Should -Be 42
            $result.registryEntries[2]._exist | Should -BeFalse
        }

        It 'Can get multiple values from offline hive using dsc config get' {
            $config_yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Reg List
  type: Microsoft.Windows/RegistryList
  properties:
    registryFilePath: '$($script:hklmHive -replace '\\', '\\')'
    registryEntries:
    - keyPath: HKLM\Software\DSCTest
      valueName: TestString
    - keyPath: HKLM\Software\DSCTest
      valueName: TestDword
    - keyPath: HKLM\Software\DSCTest
      valueName: NonExistent
"@
            $out = dsc config get --input $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $out.results.result[0].actualState.registryEntries.Count | Should -Be 3
            $out.results.result[0].actualState.registryEntries[0].valueData.String | Should -Be 'TestValue'
            $out.results.result[0].actualState.registryEntries[1].valueData.DWord | Should -Be 42
            $out.results.result[0].actualState.registryEntries[2]._exist | Should -BeFalse
        }
    }

    Context 'What-if set in offline hive' {
        BeforeEach {
            $script:hklmHive = Join-Path $TestDrive 'HKLM_whatif.hiv'
            Copy-Item (Join-Path $testHivesSource 'HKLM.hiv') -Destination $script:hklmHive
        }

        It 'What-if set reports keys to create without modifying hive' {
            $json = @{
                keyPath = 'HKLM\Software\NewKey\SubKey'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $result = registry config set -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result.keyPath | Should -Be 'HKLM\Software\NewKey\SubKey'
            $result._metadata.whatIf | Should -Not -BeNullOrEmpty

            # Verify hive was NOT modified
            $getResult = registry config get --input $json 2>$null | ConvertFrom-Json
            $getResult._exist | Should -Be $false
        }

        It 'What-if set reports value to create without modifying hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'WhatIfValue'
                valueData = @{ String = 'ShouldNotExist' }
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress -Depth 3
            $result = registry config set -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result.valueName | Should -Be 'WhatIfValue'
            $result.valueData.String | Should -Be 'ShouldNotExist'

            # Verify hive was NOT modified
            $getJson = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'WhatIfValue'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $getResult = registry config get --input $getJson 2>$null | ConvertFrom-Json
            $getResult._exist | Should -Be $false
        }

        It 'What-if set on existing value reports no changes' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestString'
                valueData = @{ String = 'TestValue' }
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress -Depth 3
            $result = registry config set -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result.keyPath | Should -Be 'HKLM\Software\DSCTest'
            $result.valueName | Should -Be 'TestString'
            # No whatIf metadata means no changes needed
            ($result.psobject.properties | Where-Object { $_.Name -eq '_metadata' } | Measure-Object).Count | Should -Be 0
        }
    }

    Context 'What-if delete in offline hive' {
        BeforeEach {
            $script:hklmHive = Join-Path $TestDrive 'HKLM_whatif_del.hiv'
            Copy-Item (Join-Path $testHivesSource 'HKLM.hiv') -Destination $script:hklmHive
        }

        It 'What-if delete value reports deletion without modifying hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestString'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $result = registry config delete -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result.keyPath | Should -Be 'HKLM\Software\DSCTest'
            $result._metadata.whatIf | Should -Match 'TestString'

            # Verify hive was NOT modified - value still readable
            $getResult = registry config get --input $json 2>$null | ConvertFrom-Json
            $getResult._exist | Should -Not -Be $false
            $getResult.valueData.String | Should -Be 'TestValue'
        }

        It 'What-if delete key reports deletion without modifying hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $result = registry config delete -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result.keyPath | Should -Be 'HKLM\Software\DSCTest'
            $result._metadata.whatIf | Should -Match 'DSCTest'

            # Verify hive was NOT modified - key still exists
            $getResult = registry config get --input $json 2>$null | ConvertFrom-Json
            $getResult._exist | Should -Not -Be $false
            $getResult.keyPath | Should -Be 'HKLM\Software\DSCTest'
        }

        It 'What-if delete non-existent value is a no-op' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'DoesNotExist'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $result = registry config delete -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result._exist | Should -Be $false
        }

        It 'What-if set with _exist false reports value deletion without modifying hive' {
            $json = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestString'
                _exist = $false
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $result = registry config set -w --input $json 2>$null | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $result.keyPath | Should -Be 'HKLM\Software\DSCTest'
            $result._metadata.whatIf | Should -Match 'TestString'

            # Verify hive was NOT modified
            $getJson = @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestString'
                registryFilePath = $script:hklmHive
            } | ConvertTo-Json -Compress
            $getResult = registry config get --input $getJson 2>$null | ConvertFrom-Json
            $getResult._exist | Should -Not -Be $false
            $getResult.valueData.String | Should -Be 'TestValue'
        }
    }
}
