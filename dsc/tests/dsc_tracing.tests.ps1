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
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: trace
              type: Test/Trace
              properties:
                level: trace
"@

        $out = (dsc -l $level config get -i $configYaml 2> $null) | ConvertFrom-Json
        $out.results[0].result.actualState.level | Should -BeExactly $level
    }

    It 'Pass-through tracing should only emit JSON for child processes' {
        $logPath = "$TestDrive/dsc_trace.log"
        $out = dsc -l info -t pass-through config get -f ../examples/groups.dsc.yaml 2> $logPath
        foreach ($line in (Get-Content $logPath)) {
            $line | Should -Not -BeNullOrEmpty
            Write-Verbose -Verbose $line
            $json = $line | ConvertFrom-Json
            $json.timestamp | Should -Not -BeNullOrEmpty
            $json.level | Should -BeIn 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'
        }
        $out | Should -BeNullOrEmpty
    }
}
