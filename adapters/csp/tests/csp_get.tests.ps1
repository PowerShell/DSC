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
            $_.schema.type | Should -BeExactly 'object'
            $_.schema.additionalProperties | Should -BeFalse
            $_.capabilities | Should -Contain 'get'
            $_.capabilities | Should -Contain 'test'
            $_.capabilities | Should -Contain 'export'
            if (@($_.schema.properties.psobject.Properties |
                Where-Object { $_.Name -notin 'InstanceID', 'ParentID', '_exist', '_inDesiredState' -and
                    !$_.Value.readOnly }).Count -eq 0) {
                $_.capabilities | Should -Not -Contain 'set'
            }
        }
    }

    It 'Returns an effective JSON Schema through DSC' {
        $schema = dsc resource schema -r $resourceType 2>$TestDrive/error.log | Out-String
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $inputJson | Test-Json -Schema $schema | Should -BeTrue
        '{"UnknownProperty":true}' | Test-Json -Schema $schema -ErrorAction Ignore | Should -BeFalse
        '{"AllowEAPCertSSO":"not-an-integer"}' | Test-Json -Schema $schema -ErrorAction Ignore | Should -BeFalse
        '{"AllowEAPCertSSO":2147483648}' | Test-Json -Schema $schema -ErrorAction Ignore | Should -BeFalse
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

    It 'Passes inputs larger than the Windows command-line limit to the adapter' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        $instanceID = 'DSC_CSP_Nonexistent_Instance'
        $json = @{
            ParentID = './Vendor/MSFT/Policy/Config'
            InstanceID = $instanceID
            StartLayout = '<LayoutModificationTemplate><!--' + ('x' * 40000) + '--></LayoutModificationTemplate>'
        } | ConvertTo-Json -Compress
        $json.Length | Should -BeGreaterThan 32767
        $inputPath = Join-Path $TestDrive 'large-input.json'
        Set-Content -LiteralPath $inputPath -Value $json -Encoding utf8
        $out = dsc resource get -r 'Microsoft.Windows.CSP/MDM_Policy_User_Config01_Start02' `
            -f $inputPath 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actual = ($out | ConvertFrom-Json).actualState
        $actual.InstanceID | Should -BeExactly $instanceID
        $actual._exist | Should -BeFalse
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
