# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config set tests' {
    BeforeEach {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "_ensure": "Absent"
        }
'@
        $json | registry config set
    }

    AfterEach {
        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_ensure": "Absent"
        }
'@
        $json | registry config set
    }

    It 'can set and remove a registry value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $out = $json | dsc resource set -r *registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.afterState.keyPath | Should -Be 'HKCU\1\2\3'
        $result.afterState.valueName | Should -Be 'Hello'
        $result.afterState.valueData.String | Should -Be 'World'
        $result.changedProperties | Should -Be @('keyPath', 'valueName', 'valueData')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3

        $out = $json | dsc resource get -r *registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.actualState.keyPath | Should -Be 'HKCU\1\2\3'
        $result.actualState.valueName | Should -Be 'Hello'
        $result.actualState.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 1

        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_ensure": "Absent"
        }
'@
        $out = $json | dsc resource set -r Microsoft.Windows/registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.afterState.keyPath | Should -BeNullOrEmpty
        $result.changedProperties | Should -Be @('keyPath')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }
}
