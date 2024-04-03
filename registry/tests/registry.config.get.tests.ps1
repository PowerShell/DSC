# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry config get tests' {
    It 'Can get a registry key' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion"
        }
'@
        $out = registry config get --input $json 
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
        $out = registry config get --input $json
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -Be 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $result.valueName | Should -Be 'ProgramFilesPath'
        $result.valueData.ExpandString | Should -Be '%ProgramFiles%'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }
}
