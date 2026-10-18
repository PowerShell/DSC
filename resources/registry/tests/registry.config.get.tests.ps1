# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry config get tests' {
    BeforeAll {
        if ($IsWindows) {
            New-Item -Path 'HKCU:\1\AclGet' -Force | Out-Null
        }
    }

    AfterAll {
        if ($IsWindows) {
            Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
        }
    }

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

    It 'Can get a registry key access control list' -Skip:(!$IsWindows) {
        $json = @{
            keyPath = 'HKCU\1\AclGet'
            accessControlList = @{ discretionary = @() }
        } | ConvertTo-Json -Depth 10 -Compress

        $result = registry config get --input $json 2>$TestDrive/error.log | ConvertFrom-Json

        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw)
        $result.keyPath | Should -BeExactly 'HKCU\1\AclGet'
        $result.accessControlList.discretionary.Count | Should -BeGreaterThan 0
        $result.accessControlList.discretionary[0].principal | Should -Match '^S-1-'
    }

    It 'Rejects an access control list request that specifies neither discretionary nor system' -Skip:(!$IsWindows) {
        $json = @{
            keyPath = 'HKCU\1\MissingAclGet'
            accessControlList = @{}
        } | ConvertTo-Json -Depth 10 -Compress

        registry config get --input $json 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Match 'Registry ACL'
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
        $result[0].level | Should -BeExactly 'TRACE'
        $result[0].fields.message | Should -BeLike 'Get Input:*'
    }
}
