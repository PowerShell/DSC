# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tracing tests' {
    It 'trace level <level> works' -TestCases @(
        @{ level = 'error' }
        # @{ level = 'WARNING' } TODO: currently no warnings are emitted
        @{ level = 'info' }
        @{ level = 'debug' }
        @{ level = 'trace' }
    ) {
        param($level)

        $logPath = "$TestDrive/dsc_trace.log"
        $null = '{}' | dsc -l $level resource get -r 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        $log | Should -BeLikeExactly "* $($level.ToUpper()) *"
    }

    It 'trace level error does not emit other levels' {
        $logPath = "$TestDrive/dsc_trace.log"
        $null = '{}' | dsc --trace-level error resource get -r 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        $log | Should -Not -BeLikeExactly "* WARNING *"
        $log | Should -Not -BeLikeExactly "* INFO *"
        $log | Should -Not -BeLikeExactly "* DEBUG *"
        $log | Should -Not -BeLikeExactly "* TRACE *"
    }

    It 'trace format plaintext does not emit ANSI' {
        $logPath = "$TestDrive/dsc_trace.log"
        $null = '{}' | dsc --trace-format plaintext resource get -r 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        $log | Should -Not -BeLikeExactly "*``[0m*"
    }

    It 'trace format json emits json' {
        $logPath = "$TestDrive/dsc_trace.log"
        $null = '{}' | dsc --trace-format json resource get -r 'DoesNotExist' 2> $logPath
        foreach ($line in (Get-Content $logPath)) {
            $trace = $line | ConvertFrom-Json -Depth 10
            $trace.timestamp | Should -Not -BeNullOrEmpty
            $trace.level | Should -BeIn 'ERROR', 'WARNING', 'INFO', 'DEBUG', 'TRACE'
            $trace.fields.message | Should -Not -BeNullOrEmpty
        }
    }

    It 'trace level <level> emits source info: <sourceExpected>' -TestCases @(
        @{ level = 'error'; sourceExpected = $false }
        @{ level = 'warning'; sourceExpected = $false }
        @{ level = 'info'; sourceExpected = $false }
        @{ level = 'debug'; sourceExpected = $true }
        @{ level = 'trace'; sourceExpected = $true }
    ) {
        param($level, $sourceExpected)

        $logPath = "$TestDrive/dsc_trace.log"
        $null = '{}' | dsc -l $level resource get -r 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        if ($sourceExpected) {
            $log | Should -BeLike "*dsc*: *"
        } else {
            $log | Should -Not -BeLike "*dsc*: *"
        }
    }
}
