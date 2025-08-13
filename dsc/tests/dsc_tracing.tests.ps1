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
        $null = dsc -l $level resource get -r 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        $log | Should -BeLikeExactly "* $($level.ToUpper()) *"
    }

    It 'trace level error does not emit other levels' {
        $logPath = "$TestDrive/dsc_trace.log"
        $null = dsc --trace-level error resource list 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        $log | Should -Not -BeLikeExactly "* WARNING *"
        $log | Should -Not -BeLikeExactly "* INFO *"
        $log | Should -Not -BeLikeExactly "* DEBUG *"
        $log | Should -Not -BeLikeExactly "* TRACE *"
    }

    It 'trace format plaintext does not emit ANSI' {
        $logPath = "$TestDrive/dsc_trace.log"
        $null = dsc --trace-format plaintext resource list 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        $log | Should -Not -BeLikeExactly "*``[0m*"
    }

    It 'trace format json emits json' {
        $logPath = "$TestDrive/dsc_trace.log"
        $null = dsc --trace-format json resource list 'DoesNotExist' 2> $logPath
        foreach ($line in (Get-Content $logPath)) {
            $trace = $line | ConvertFrom-Json -Depth 10
            $trace.timestamp | Should -Not -BeNullOrEmpty
            $trace.level | Should -BeIn 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'
            $trace.fields.message | Should -Not -BeNullOrEmpty
        }
    }

    It 'trace level <level> emits source info: <sourceExpected>' -TestCases @(
        @{ level = 'error'; sourceExpected = $false }
        @{ level = 'warn'; sourceExpected = $false }
        @{ level = 'info'; sourceExpected = $false }
        @{ level = 'debug'; sourceExpected = $true }
        @{ level = 'trace'; sourceExpected = $true }
    ) {
        param($level, $sourceExpected)

        $logPath = "$TestDrive/dsc_trace.log"
        $null = dsc -l $level resource list 'DoesNotExist' 2> $logPath
        $log = Get-Content $logPath -Raw
        if ($sourceExpected) {
            $log | Should -BeLike "*dsc_lib*: *"
        } else {
            $log | Should -Not -BeLike "*dsc_lib*: *"
        }
    }

    It 'trace level <level> is passed to resource' -TestCases @(
        @{ level = 'error' }
        @{ level = 'warn' }
        @{ level = 'info' }
        @{ level = 'debug' }
        @{ level = 'trace' }
    ) {
        param($level)

        $configYaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: trace
              type: Test/Trace
              properties:
                level: trace
"@

        $out = (dsc -l $level config get -i $configYaml 2> $TestDrive/error.log) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log | Out-String)
        $out.results[0].result.actualState.level | Should -BeExactly $level -Because ($out | Out-String)
    }

    It 'Pass-through tracing should only emit JSON for child processes' {
        $logPath = "$TestDrive/dsc_trace.log"
        $out = dsc -l trace -t pass-through config get -f "$PSScriptRoot/../examples/groups.dsc.yaml" 2> $logPath
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $logPath -Raw)
        $foundPID = $false
        $foundTarget = $false
        $foundLineNumber = $false
        foreach ($line in (Get-Content $logPath)) {
            $line | Should -Not -BeNullOrEmpty
            $json = $line | ConvertFrom-Json
            $json.timestamp | Should -Not -BeNullOrEmpty -Because "Line: $line"
            $json.level | Should -BeIn 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'
            $json.fields.message | Should -Not -BeNullOrEmpty -Because "Line: $line"
            if ($json.fields.pid) {
                $json.fields.pid | Should -BeGreaterThan 0 -Because "Line: $line"
                $foundPID = $true
            }
            if ($json.fields.target) {
                $foundTarget = $true
            }
            if ($json.fields.line_number) {
                $json.fields.line_number | Should -BeGreaterThan 0 -Because "Line: $line"
                $foundLineNumber = $true
            }
        }
        $foundTarget | Should -BeTrue -Because "No target found in log"
        $foundLineNumber | Should -BeTrue -Because "No line number found in log"
        $foundPID | Should -BeTrue -Because "No PID found in log"
        $out | Should -Not -BeNullOrEmpty
    }
}
