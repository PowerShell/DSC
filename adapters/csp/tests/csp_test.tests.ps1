# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'CSP test' -Skip:(!$IsWindows) {
    BeforeAll {
        . (Join-Path $PSScriptRoot 'CspTestSetup.ps1')
    }

    AfterAll {
        . (Join-Path $PSScriptRoot 'CspTestCleanup.ps1')
    }

    It 'Tests an observed current-user state without comparing unspecified properties' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        $out = $inputJson | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $state = ($out | ConvertFrom-Json -AsHashtable).actualState
        $state.Remove('AllowEAPCertSSO')
        $json = $state | ConvertTo-Json -Compress
        $out = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result = $out | ConvertFrom-Json
        $result.inDesiredState | Should -BeTrue
        $result.differingProperties | Should -HaveCount 0
    }

    It 'Reports an existence mismatch in the explicit property diff' {
        if ($isSystem) {
            Set-ItResult -Skipped -Because 'This test requires the current user rather than LocalSystem.'
            return
        }
        $out = $inputJson | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $state = ($out | ConvertFrom-Json -AsHashtable).actualState
        $state.Remove('AllowEAPCertSSO')
        $state._exist = !$state._exist
        $out = ($state | ConvertTo-Json -Compress) |
            dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result = $out | ConvertFrom-Json
        $result.inDesiredState | Should -BeFalse
        $result.differingProperties | Should -Contain '_exist'
    }
}
