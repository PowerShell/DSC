# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'CSP set' -Skip:(!$IsWindows) {
    BeforeAll {
        . (Join-Path $PSScriptRoot 'CspTestSetup.ps1')
    }

    AfterAll {
        . (Join-Path $PSScriptRoot 'CspTestCleanup.ps1')
    }

    It 'Rejects invalid writable values without changing state' -ForEach @(
        @{ Value = '"not-an-integer"' }
        @{ Value = '2147483648' }
        @{ Value = '1.5' }
    ) {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        $before = $inputJson | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $json = '{"ParentID":"./Vendor/MSFT/Policy/Config","InstanceID":"Authentication","AllowEAPCertSSO":' + $Value + '}'
        $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Not -Be 0
        $after = $inputJson | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $after | Should -BeExactly $before
    }

    It 'Treats deleting an already absent instance as an idempotent operation' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        $absentJson = '{"ParentID":"./Vendor/MSFT/Policy/Config","InstanceID":"DSC_CSP_Nonexistent_Instance","_exist":false}'
        $before = $absentJson | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        ($before | ConvertFrom-Json).actualState._exist | Should -BeFalse
        $out = $absentJson | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        ($out | ConvertFrom-Json).afterState._exist | Should -BeFalse
    }
}
