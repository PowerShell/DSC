# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'exit code tests' {
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
}
