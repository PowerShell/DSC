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

    It 'Can whatif delete an existing value using _exist is false' -Skip:(!$IsWindows) {
        $set_json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        registry config set --input $set_json | Out-Null

        $whatif_delete_value = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "_exist": false
        }
'@
        $result = registry config set -w --input $whatif_delete_value 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result.valueName | Should -Be 'Hello'
        $result._metadata.whatIf | Should -Match "Would delete value 'Hello'"
    }

    It 'Can whatif delete an existing subkey using _exist is false' -Skip:(!$IsWindows) {
        $set_key = @'
        {
            "keyPath": "HKCU\\1\\2\\3"
        }
'@
        registry config set --input $set_key | Out-Null

        $whatif_delete_key = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "_exist": false
        }
'@
        $result = registry config set -w --input $whatif_delete_key 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result._metadata.whatIf | Should -Match "Would delete subkey '3'"
        # For delete what-if, payload should only include keyPath (and optionally valueName when deleting a value)
        ($result.psobject.properties | Where-Object { $_.Name -ne '_metadata' } | Measure-Object).Count | Should -Be 1
    }

    It 'Can whatif delete an existing subkey' -Skip:(!$IsWindows) {
        $set_key = @'
        {
            "keyPath": "HKCU\\1\\2\\3"
        }
'@
        registry config set --input $set_key | Out-Null

        $whatif_delete_key = @'
        {
            "keyPath": "HKCU\\1\\2\\3"
        }
'@
        $result = registry config delete -w --input $whatif_delete_key 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.keyPath | Should -Be 'HKCU\1\2\3'
        $result._metadata.whatIf | Should -Match "Would delete subkey '3'"
        # For delete what-if, payload should only include keyPath (and optionally valueName when deleting a value)
        ($result.psobject.properties | Where-Object { $_.Name -ne '_metadata' } | Measure-Object).Count | Should -Be 1
    }

    It 'Can whatif multiple keys in a registryKeys array' -Skip:(!$IsWindows) {
        $json = @'
        {
            "registryKeys": [
                {
                    "keyPath": "HKCU\\1\\A",
                    "valueName": "First",
                    "valueData": { "String": "alpha" }
                },
                {
                    "keyPath": "HKCU\\1\\B",
                    "valueName": "Second",
                    "valueData": { "DWord": 42 }
                }
            ]
        }
'@
        $get_before = registry config get --input $json 2>$null
        $result = registry config set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.registryKeys.Count | Should -Be 2
        $result.registryKeys[0].keyPath | Should -Be 'HKCU\1\A'
        $result.registryKeys[0].valueData.String | Should -Be 'alpha'
        $result.registryKeys[0]._metadata.whatIf | Should -Not -BeNullOrEmpty
        $result.registryKeys[1].keyPath | Should -Be 'HKCU\1\B'
        $result.registryKeys[1].valueData.DWord  | Should -Be 42
        $result.registryKeys[1]._metadata.whatIf | Should -Not -BeNullOrEmpty
        $get_after = registry config get --input $json 2>$null
        $get_before | Should -EQ $get_after
    }
}
