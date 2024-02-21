# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for runcommandonset set' {
    BeforeAll {
        $oldPath = $env:DSC_RESOURCE_PATH
        $env:DSC_RESOURCE_PATH = Join-Path $PSScriptRoot ".."
        $yaml = @"
executable: pwsh.exe
arguments:
- -Command
- echo hello world
"@
    }

    AfterEach {
        if (Test-Path $TestDrive/output.txt) {
            Remove-Item -Path $TestDrive/output.txt
        }
    }

    AfterAll {
        $env:DSC_RESOURCE_PATH = $oldPath
    }

    It 'Input can be sent to the resource via stdin json' {
        $json = @"
        {
            "executable": "pwsh.exe",
            "arguments": ["-Command", "echo hello world"],
        }
"@

        $json | dsc resource set -r Microsoft/RunCommandOnSet > $TestDrive/output.txt
        # TODO: test output once DSC PR to capture it is merged
        $LASTEXITCODE | Should -Be 0
    }

    It 'Input can be sent to the resource via stdin yaml' {
        $yaml | dsc resource set -r Microsoft/RunCommandOnSet > $TestDrive/output.txt
        # TODO: test output once DSC PR to capture it is merged
        $LASTEXITCODE | Should -Be 0
    }

    It 'STDOUT captured via STDERR when calling resource directly' {
        $yaml | runcommandonset set 2> $TestDrive/output.txt
        $actual = Get-Content -Path $TestDrive/output.txt
        $actual | Should -Contain 'Stdout: hello'
        $actual | Should -Contain 'world'
        $LASTEXITCODE | Should -Be 0
    }

    It 'STDERR captured when calling resource directly with invalid args' {
        $json = runcommandonset set -e pwsh.exe -a "echo hello world" 2> $TestDrive/output.txt
        $stdout = $json | ConvertFrom-Json
        $stdout.exit_code | Should -Be 64
        $expected = "Stderr: The argument 'echo hello world' is not recognized as the name of a script file. Check the spelling of the name, or if a path was included, verify that the path is correct and try again."
        $stderr = Get-Content -Path $TestDrive/output.txt
        $stderr | Should -Contain $expected
        $LASTEXITCODE | Should -Be 0
    }
}
