# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for runcommandonset set' {
    BeforeAll {
        $oldPath = $env:DSC_RESOURCE_PATH
        $env:DSC_RESOURCE_PATH = Join-Path $PSScriptRoot ".."
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
        get-content $testdrive/output.txt | write-host
        $LASTEXITCODE | Should -Be 0
    }

    It 'Input can be sent to the resource via stdin yaml' {
        $yaml = @"
executable: pwsh.exe
arguments:
- -Command
- echo hello world
"@

        $yaml | dsc -l info resource set -r Microsoft/RunCommandOnSet > $TestDrive/output.txt
        $output = get-content $testdrive/output.txt | write-host
        $LASTEXITCODE | Should -Be 0
    }
}
