# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'registry config set tests' {
    AfterEach {
        if ($IsWindows) {
            Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
        }
    }

    It 'Can set a deeply nested key and value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $out = registry config set --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeNullOrEmpty
        $result = registry config get --input $json 2>$null | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3

        $out = registry config get --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }

    It 'delete called when _exist is false' -Skip:(!$IsWindows) {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'reg'
                    type = 'Microsoft.Windows/Registry'
                    properties = @{
                        keyPath = 'HKCU\1\2'
                        valueName = 'Test'
                        valueData = @{
                            String = 'Test'
                        }
                        _exist = $true
                    }
                }
            )
        }

        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10)
        $LASTEXITCODE | Should -Be 0

        $config.resources[0].properties._exist = $false
        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false

        Get-ItemProperty -Path 'HKCU:\1\2' -Name 'Test' -ErrorAction Ignore | Should -BeNullOrEmpty

        $config.resources[0].properties.valueName = $null
        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false

        Get-Item -Path 'HKCU:\1\2' -ErrorAction Ignore | Should -BeNullOrEmpty
    }

    It 'Can set value without data' -Skip:(!$IsWindows) {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Key
              type: Microsoft.Windows/Registry
              properties:
                keyPath: 'HKCU\1'
                valueName: Test
                _exist: true
'@

        $out = dsc config set -i $configYaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState.keyPath | Should -BeExactly 'HKCU\1'
        $out.results[0].result.afterState.valueName | Should -BeExactly 'Test'
        $out.results[0].result.afterState.valueData | Should -BeNullOrEmpty

        $out = dsc config get -i $configYaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.keyPath | Should -BeExactly 'HKCU\1'
        $out.results[0].result.actualState.valueName | Should -BeExactly 'Test'
        $out.results[0].result.actualState.valueData | Should -BeNullOrEmpty

        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    It 'Should succeed when _exist is false and value does not exist' -Skip:(!$IsWindows) {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'reg'
                    type = 'Microsoft.Windows/Registry'
                    properties = @{
                        keyPath = 'HKCU'
                        valueName = 'Test'
                        valueData = @{
                            String = 'Test'
                        }
                        _exist = $false
                    }
                }
            )
        }

        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false

        Get-ItemProperty -Path 'HKCU:\1\2' -Name 'Test' -ErrorAction Ignore | Should -BeNullOrEmpty
    }

    It 'Should succeed when _exist is false and key does not exist' -Skip:(!$IsWindows) {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'reg'
                    type = 'Microsoft.Windows/Registry'
                    properties = @{
                        keyPath = 'HKCU\1'
                        _exist = $false
                    }
                }
            )
        }

        $out = dsc config set -i ($config | ConvertTo-Json -Depth 10) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.afterState._exist | Should -Be $false
    }

    It 'Can delete value from system-protected key with minimal permissions' -Skip:(!$IsWindows) {
        $testKeyPath = 'HKLM:\Software\Policies\Microsoft\Windows\Appx'
        if (-not (Test-Path $testKeyPath)) {
            Set-ItResult -Skipped -Because "Test key path '$testKeyPath' does not exist"
            return
        }

        $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
        $isElevated = $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        
        if (-not $isElevated) {
            Set-ItResult -Skipped -Because "Test requires elevated privileges"
            return
        }

        $setJson = @'
        {
            "keyPath": "HKEY_LOCAL_MACHINE\\Software\\Policies\\Microsoft\\Windows\\Appx",
            "valueName": "DSCTestValue",
            "valueData": {
                "String": "TestData"
            }
        }
'@
        $out = registry config set --input $setJson 2>$null
        $LASTEXITCODE | Should -Be 0

        $getJson = @'
        {
            "keyPath": "HKEY_LOCAL_MACHINE\\Software\\Policies\\Microsoft\\Windows\\Appx",
            "valueName": "DSCTestValue"
        }
'@
        $result = registry config get --input $getJson 2>$null | ConvertFrom-Json
        $result.valueName | Should -Be 'DSCTestValue'
        $result.valueData.String | Should -Be 'TestData'

        $deleteJson = @'
        {
            "keyPath": "HKEY_LOCAL_MACHINE\\Software\\Policies\\Microsoft\\Windows\\Appx",
            "valueName": "DSCTestValue"
        }
'@
        $out = registry config delete --input $deleteJson 2>$null
        $LASTEXITCODE | Should -Be 0

        $result = registry config get --input $getJson 2>$null | ConvertFrom-Json
        $result._exist | Should -Be $false
        $result.valueData | Should -BeNullOrEmpty
    }
}
