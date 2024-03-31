# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'registry config set tests' {
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

    It 'Can set a key to be absent' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_exist": false
        }
'@
        $out = registry config set --input $json
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -BeExactly 'HKCU\1'
        $result._exist | Should -Be $false
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }
}
