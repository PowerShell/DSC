# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry config get tests' {
    It 'Can get a registry key' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion"
        }
'@
        $out = registry config get --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 1
    }

    It 'Can get a registry value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion",
            "valueName": "ProgramFilesPath"
        }
'@
        $out = registry config get --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $result.valueName | Should -Be 'ProgramFilesPath'
        $result.valueData.ExpandString | Should -Be '%ProgramFiles%'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }

    It 'Traces should be JSON' -Skip:(!$IsWindows) {
        # keyPath should return Access Denied
        $json = @'
        {
            "keyPath": "HKLM\\SYSTEM\\CurrentControlSet\\Control"
        }
'@
        $out = registry config get --input $json 2>&1
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result[0].level | Should -BeExactly 'DEBUG'
        $result[0].fields.message | Should -BeLike 'Get Input:*'
    }

    It 'Can get multiple registry keys via the registryKeys array' -Skip:(!$IsWindows) {
        $json = @'
        {
            "registryKeys": [
                {
                    "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion",
                    "valueName": "ProgramFilesPath"
                },
                {
                    "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion"
                }
            ]
        }
'@
        $out = registry config get --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.registryKeys.Count | Should -Be 2
        $result.registryKeys[0].keyPath  | Should -Be 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $result.registryKeys[0].valueName | Should -Be 'ProgramFilesPath'
        $result.registryKeys[0].valueData.ExpandString | Should -Be '%ProgramFiles%'
        $result.registryKeys[1].keyPath  | Should -Be 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $result.registryKeys[1].valueName | Should -BeNullOrEmpty
    }

    It 'Returns _exist=false per item when a key/value is missing in the registryKeys array' -Skip:(!$IsWindows) {
        $json = @'
        {
            "registryKeys": [
                { "keyPath": "HKCU\\DSCArrayTestDoesNotExist" },
                { "keyPath": "HKCU\\Software", "valueName": "DSCArrayTestDoesNotExist" }
            ]
        }
'@
        $out = registry config get --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.registryKeys.Count | Should -Be 2
        $result.registryKeys[0]._exist | Should -Be $false
        $result.registryKeys[1]._exist | Should -Be $false
    }

    It 'Returns the registryKeys wrapper shape when input used the wrapper, even with one item' -Skip:(!$IsWindows) {
        $json = @'
        {
            "registryKeys": [
                { "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion", "valueName": "ProgramFilesPath" }
            ]
        }
'@
        $out = registry config get --input $json 2>$null
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.PSObject.Properties.Name | Should -Contain 'registryKeys'
        $result.registryKeys.Count | Should -Be 1
        $result.registryKeys[0].valueName | Should -Be 'ProgramFilesPath'
    }
}
