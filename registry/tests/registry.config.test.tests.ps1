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
        $result.keyPath | Should -BeNullOrEmpty
        $result._exist | Should -Be $exist
        ($result.psobject.properties | Measure-Object).Count | Should -Be 4
    }

    It 'Can report failure if a registry key <test>' -Skip:(!$IsWindows) -TestCases @(
        @{ test = 'exists and absent'; exist = 'false'; key = 'CurrentVersion'; expectedKey = 'HKLM\Software\Microsoft\Windows NT\CurrentVersion' }
        @{ test = 'does not exist and present'; exist = 'true'; key = 'DoesNotExist'; expectedKey = '' }
    ){
        param($exist, $key, $expectedKey)
        $json = @"
        {
          "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\$key",
          "_exist": $ensure
        }
"@
        $out = $json | registry config test
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.keyPath | Should -BeExactly $expectedKey
        $result._inDesiredState | Should -Be $false
        $result._exist | Should -Be $exist
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
