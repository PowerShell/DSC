# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'registry config whatif tests' {
    BeforeAll {
        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    AfterEach {
        Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
    }

    It 'Can whatif a new deeply nested key' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3"
        }
'@
        $get_before = registry config get --input $json
        $result = registry config set -w --input $json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $get_after = registry config get --input $json
        $get_before | Should -EQ $get_after
    }

    It 'Can whatif a new deeply nested key and value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $result = registry config set -w --input $json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
    }

    It 'Can whatif an existing key with new value' -Skip:(!$IsWindows) {
        $set_json = @'
        {
            "keyPath": "HKCU\\1\\2"
        }
'@
        registry config set --input $set_json
        $whatif_json = @'
            {
                "keyPath": "HKCU\\1\\2",
                "valueName": "Hello",
                "valueData": {
                    "String": "World"
                }
            }
'@
        $result = registry config set -w --input $whatif_json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
    }

    It 'Can whatif an existing deeply nested key and value' -Skip:(!$IsWindows) {
        $set_json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        registry config set --input $set_json
        $whatif_json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World-WhatIf"
            }
        }
'@
        $result = registry config set -w --input $whatif_json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World-WhatIf'
    }

    It 'Can whatif an existing key with nested values' -Skip:(!$IsWindows) {
        $set_json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        registry config set --input $set_json
        $set_json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Foo",
            "valueData": {
                "String": "Bar"
            }
        }
'@
        registry config set --input $set_json
        $whatif_json = @'
        {
            "keyPath": "HKCU\\1\\2"
        }
'@
        $result = registry config set -w --input $whatif_json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 1
    }
}
