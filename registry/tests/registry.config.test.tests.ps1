# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'registry config test tests' {
    It 'Can test a registry key <test>' -Skip:(!$IsWindows) -TestCases @(
        @{ test = 'exists and present'; exist = 'true'; key = 'CurrentVersion' }
        @{ test = 'does not exist and absent'; exist = 'false'; key = 'DoesNotExist' }
    ){
        param($exist, $key)
        $json = @"
        {
          "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\$key",
          "_exist": $exist
        }
"@
        $out = $json | registry config test
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -BeExactly "HKLM\Software\Microsoft\Windows NT\$key"
        $result._exist | Should -Be $exist
        ($result.psobject.properties | Measure-Object).Count | Should -Be 4
    }

    It 'Can report failure if a registry key <test>' -Skip:(!$IsWindows) -TestCases @(
        @{ test = 'exists'; exist = 'false'; expectedExist = $true; key = 'CurrentVersion' }
        @{ test = 'does not exist'; exist = 'true'; expectedExist = $false; key = 'DoesNotExist' }
    ){
        param($exist, $expectedExist, $key)
        $json = @"
        {
          "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\$key",
          "_exist": $exist
        }
"@
        $out = $json | registry config test
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -BeExactly "HKLM\Software\Microsoft\Windows NT\$key"
        $result._inDesiredState | Should -Be $false
        $result._exist | Should -BeExactly $expectedExist
        ($result.psobject.properties | Measure-Object).Count | Should -Be 4
    }

    It 'Can test a registry value exists' -Skip:(!$IsWindows) {
        $json = @"
        {
          "keyPath": "HKLM\\Software\\Microsoft\\Windows\\CurrentVersion",
          "valueName": "ProgramFilesPath",
          "_exist": true
        }
"@
        $out = $json | registry config test
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -BeExactly 'HKLM\Software\Microsoft\Windows\CurrentVersion'
        $result.valueName | Should -BeExactly 'ProgramFilesPath'
        $result.valueData.ExpandString | Should -BeExactly '%ProgramFiles%'
        $result._inDesiredState | Should -Be $true
        $result._exist | Should -Be $true
        ($result.psobject.properties | Measure-Object).Count | Should -Be 6
    }
}
