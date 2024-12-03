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
        $out = registry config set --input $json
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeNullOrEmpty
        $result = registry config get --input $json | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3

        $out = registry config get --input $json
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }

    It 'delete called when _exist is false' -Skip:(!$IsWindows) {
        $config = @{
            '$schema' = 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
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
}
