# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'CSP get and discovery' -Skip:(!$IsWindows) {
    BeforeAll {
        . (Join-Path $PSScriptRoot 'CspTestSetup.ps1')
    }

    AfterAll {
        . (Join-Path $PSScriptRoot 'CspTestCleanup.ps1')
    }

    It 'Discovers bridge classes with schemas and capability-specific metadata' {
        $resources = @(dsc resource list --adapter $adapterType 2>$TestDrive/error.log | ConvertFrom-Json)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources.Count | Should -BeGreaterThan 0
        $resources | ForEach-Object {
            $_.type | Should -Match '^Microsoft\.Windows\.CSP/MDM_\w+$'
            $_.requireAdapter | Should -BeExactly $adapterType
            $_.schema.embedded.additionalProperties | Should -BeFalse
            $_.capabilities | Should -Contain 'get'
            $_.capabilities | Should -Contain 'test'
            $_.capabilities | Should -Contain 'export'
            if (@($_.schema.embedded.properties.psobject.Properties |
                Where-Object { $_.Name -notin 'InstanceID', 'ParentID', '_exist', '_inDesiredState' -and
                    !$_.Value.readOnly }).Count -eq 0) {
                $_.capabilities | Should -Not -Contain 'set'
            }
        }
    }

    It 'Gets the same current-user instance as the native bridge' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        $native = @(Get-CimInstance -Namespace 'root\cimv2\mdm\dmmap' `
            -ClassName 'MDM_Policy_User_Config01_Authentication02' `
            -Filter "ParentID='./Vendor/MSFT/Policy/Config' and InstanceID='Authentication'" -ErrorAction Stop)
        $out = $inputJson | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actual = ($out | ConvertFrom-Json).actualState
        $actual.ParentID | Should -BeExactly './Vendor/MSFT/Policy/Config'
        $actual.InstanceID | Should -BeExactly 'Authentication'
        $actual._exist | Should -Be ($native.Count -eq 1)
        if ($native.Count -eq 1) {
            $actual.AllowEAPCertSSO | Should -Be $native[0].AllowEAPCertSSO
        }
    }

    It 'Rejects missing keys rather than getting an arbitrary instance' {
        '{}' | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive/error.log | Should -Match 'key property'
    }

    It 'Reports the LocalSystem requirement for a device class' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'The caller already is LocalSystem.'
            return
        }
        $json = '{"ParentID":"./Vendor/MSFT/Policy/Config","InstanceID":"WiFi"}'
        $json | dsc resource get -r 'Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02' -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive/error.log | Should -Match 'LocalSystem'
    }
}
