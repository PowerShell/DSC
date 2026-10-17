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

    It 'Can whatif setting an access control list without changing it' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\AclWhatIf' -Force | Out-Null
        $getJson = @{
            keyPath = 'HKCU\1\AclWhatIf'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress
        $before = (registry config get --input $getJson 2>$TestDrive/error.log |
            ConvertFrom-Json).accessControlList

        $json = @{
            keyPath = 'HKCU\1\AclWhatIf'
            accessControlList = @{
                discretionary          = @(
                    @{
                        principal  = $env:USERNAME
                        type       = 'allow'
                        accessMask = @('fullControl')
                    }
                )
                discretionaryProtected = $true
            }
        } | ConvertTo-Json -Depth 10 -Compress

        $result = registry config set -w --input $json 2>$TestDrive/error.log | ConvertFrom-Json

        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result.accessControlList.discretionaryProtected | Should -BeTrue
        $result._metadata.whatIf | Should -Contain "Would set ACL on key 'HKCU\1\AclWhatIf'"

        $after = (registry config get --input $getJson 2>$TestDrive/error.log |
            ConvertFrom-Json).accessControlList
        $after.discretionaryProtected | Should -Be $before.discretionaryProtected
        ($after.discretionary | ConvertTo-Json -Depth 10 -Compress) |
            Should -BeExactly ($before.discretionary | ConvertTo-Json -Depth 10 -Compress)
    }

    It 'Can whatif setting a value and merging an ACL on an existing key, returning the effective merged ACL' -Skip:(!$IsWindows) {
        New-Item -Path 'HKCU:\1\AclWhatIfValueName' -Force | Out-Null
        $initial = @{
            keyPath           = 'HKCU\1\AclWhatIfValueName'
            accessControlList = @{
                discretionary = @(
                    @{ principal = $env:USERNAME; type = 'allow'; accessMask = @('fullControl') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        registry config set --input $initial 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # The current user's SID, resolved so it can be compared against the merged output.
        $currentUserSid = ([Security.Principal.NTAccount]::new($env:USERNAME)).Translate(
            [Security.Principal.SecurityIdentifier]).Value
        $anonymousSid = 'S-1-5-7'

        $whatIfJson = @{
            keyPath           = 'HKCU\1\AclWhatIfValueName'
            valueName         = 'Hello'
            valueData         = @{ String = 'World' }
            accessControlList = @{
                discretionary = @(
                    @{ principal = $anonymousSid; type = 'deny'; accessMask = @('read') }
                )
                _purge        = $false
            }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config set -w --input $whatIfJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # Since the key already exists, the what-if preview must compute and return the same
        # effective, normalized/merged ACL that an actual Set would apply -- both the
        # pre-existing entry and the newly merged one -- not the raw, unmerged request.
        $principals = $result.accessControlList.discretionary.principal
        $principals | Should -Contain $currentUserSid
        $principals | Should -Contain $anonymousSid

        # What-if must not have actually written the value or changed the ACL.
        $afterGet = registry config get --input $initial 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $afterGet.accessControlList.discretionary.Count | Should -Be 1
    }

    It 'Can whatif setting a value and ACL on a missing key, returning the raw (unmerged) request' -Skip:(!$IsWindows) {
        $whatIfJson = @{
            keyPath           = 'HKCU\1\AclWhatIfMissingKey'
            valueName         = 'Hello'
            valueData         = @{ String = 'World' }
            accessControlList = @{
                discretionary = @(
                    @{ principal = 'S-1-5-7'; type = 'deny'; accessMask = @('read') }
                )
            }
        } | ConvertTo-Json -Depth 10 -Compress
        $result = registry config set -w --input $whatIfJson 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)

        # The key doesn't exist, so the preview falls back to reporting the raw request rather
        # than attempting (and failing) to read a nonexistent key's current ACL.
        $result.accessControlList.discretionary.Count | Should -Be 1
        $result.accessControlList.discretionary[0].principal | Should -BeExactly 'S-1-5-7'
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


    It 'Removing non-existing key' -Skip:(!$IsWindows) {
        $after_config_yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Reg 1
              type: Microsoft.Windows/Registry
              properties:
                keyPath: HKCU\1\2\NonExisting
                _exist: false
'@
        $out = dsc -l trace config set --what-if --input $after_config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $out.results.result[0].afterState.keyPath | Should -BeExactly 'HKCU\1\2\NonExisting'
        $out.results.executionInformation.whatIf[0] | Should -Match "Key 'HKCU\\1\\2\\NonExisting' not found, would do nothing"
    }    
}
