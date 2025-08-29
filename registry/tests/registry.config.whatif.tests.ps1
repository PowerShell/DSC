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
        $null = registry config set -w --input $json 2>$null
        $get_before = registry config get --input $json 2>$null
        $result = registry config set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result._metadata.whatIf[0] | Should -Match '.*1.*'
        $result._metadata.whatIf[1] | Should -Match '.*2.*'
        $result._metadata.whatIf[2] | Should -Match '.*3.*'
        $get_after = registry config get --input $json 2>$null
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
        $result = registry config set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        $result._metadata.whatIf[0] | Should -Match '.*1.*'
        $result._metadata.whatIf[1] | Should -Match '.*2.*'
        $result._metadata.whatIf[2] | Should -Match '.*3.*'
    }

    It 'Can whatif an existing key with new value' -Skip:(!$IsWindows) {
        $set_json = @'
        {
            "keyPath": "HKCU\\1\\2"
        }
'@
        registry config set --input $set_json 2>$null
        $whatif_json = @'
            {
                "keyPath": "HKCU\\1\\2",
                "valueName": "Hello",
                "valueData": {
                    "String": "World"
                }
            }
'@
        $result = registry config set -w --input $whatif_json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
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
        registry config set --input $set_json 2>$null
        $whatif_json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World-WhatIf"
            }
        }
'@
        $result = registry config set -w --input $whatif_json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result.valueData.String | Should -Be 'World-WhatIf'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
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
        registry config set --input $set_json 2>$null
        $set_json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Foo",
            "valueData": {
                "String": "Bar"
            }
        }
'@
        registry config set --input $set_json 2>$null
        $whatif_json = @'
        {
            "keyPath": "HKCU\\1\\2"
        }
'@
        $result = registry config set -w --input $whatif_json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 1
    }
}
