# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'exit code tests' {
    BeforeAll {
        $env:DSC_TRACE_LEVEL = 'error'
    }

    AfterAll {
        $env:DSC_TRACE_LEVEL = $null
    }

    It 'non-zero exit code in manifest has corresponding message' {
        dsc resource get -r Test/ExitCode --input "{ exitCode: 8 }" 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Placeholder from manifest for exit code 8'
    }
    It 'non-zero exit code not in manifest has generic message' {
        dsc resource get -r Test/ExitCode --input "{ exitCode: 1 }" 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'exit code 1'
    }
    It 'success exit code executes without error' {
        $result = dsc resource get -r Test/ExitCode --input "{ exitCode: 0 }" | ConvertFrom-Json
        $result.actualState.exitCode | Should -Be 0
        $LASTEXITCODE | Should -Be 0
    }
    It 'Exiting early due to broken pipe is a success' {
        $out = dsc resource list | Select-Object -First 1 | ConvertFrom-Json
        $out.Count | Should -Be 1
        $LASTEXITCODE | Should -Be 0
    }
}
