# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'CSP export' -Skip:(!$IsWindows) {
    BeforeAll {
        . (Join-Path $PSScriptRoot 'CspTestSetup.ps1')
    }

    AfterAll {
        . (Join-Path $PSScriptRoot 'CspTestCleanup.ps1')
    }

    It 'Exports the instances available to the current user' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        $native = @(Get-CimInstance -Namespace 'root\cimv2\mdm\dmmap' `
            -ClassName 'MDM_Policy_User_Config01_Authentication02' -ErrorAction Stop)
        $out = dsc resource export -r $resourceType 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources = @(($out | ConvertFrom-Json).resources)
        $resources.Count | Should -Be $native.Count
        $resources | ForEach-Object {
            $_.type | Should -BeExactly $resourceType
            $_.properties._exist | Should -BeTrue
            $_.properties.psobject.Properties.Name | Should -Not -Contain '__PATH'
        }
    }

    It 'Accepts a partial key filter and rejects non-key filters' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        '{"InstanceID":"Authentication"}' |
            dsc resource export -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        '{"AllowEAPCertSSO":1}' |
            dsc resource export -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Not -Be 0
    }
}
