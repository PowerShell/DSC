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

    It 'Traces should be JSON' -Skip:(!$IsWindows) {
        # keyPath should return Access Denied
        $json = @'
        {
            "keyPath": "HKLM\\SYSTEM\\CurrentControlSet\\Control\\SecurePipeServers\\Winreg"
        }
'@
        $out = registry config get --input $json 2>&1
        $LASTEXITCODE | Should -Be 3
        $result = $out | ConvertFrom-Json
        $result.level | Should -BeExactly 'ERROR'
        $result.fields.message | Should -BeLike '*Permission denied*'
    }
}
