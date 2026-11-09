# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.System/Process get' {
    It 'Gets the current process by PID' {
        $json = @{ pid = $PID } | ConvertTo-Json -Compress
        $out = $json | dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $state = ($out | ConvertFrom-Json).actualState
        $state.pid | Should -Be $PID
        $state.name | Should -Not -BeNullOrEmpty
        $state.cmdline | Should -BeOfType [string]
        @($state.cmdline).Count | Should -BeGreaterThan 0
    }

    It 'Rejects invalid get input <InputJson>' -ForEach @(
        @{ InputJson = '{}' }
        @{ InputJson = '{"name":""}' }
        @{ InputJson = '{"name":"*"}' }
        @{ InputJson = '{"pid":-1}' }
        @{ InputJson = '{"pid":4294967296}' }
        @{ InputJson = '{"unknown":true}' }
    ) {
        $InputJson | dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive\error.log | Should -Not -BeNullOrEmpty
    }

    It 'Reports an error when the process does not exist' {
        '{"name":"dsc-no-such-process-cf04a8a2"}' |
            dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive\error.log | Should -Match 'No matching process'
    }
}
