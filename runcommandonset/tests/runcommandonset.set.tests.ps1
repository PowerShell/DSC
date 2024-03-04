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

    It 'Input for executable and arguments can be sent to the resource' {
        $input_json = @"
        {
            "executable": "pwsh",
            "arguments": ["-Command", "echo hello world"]
        }
"@
        $input_json | dsc resource set -r Microsoft.DSC.Transitional/RunCommandOnSet
        # TODO: test output once DSC PR to capture it is merged
        $LASTEXITCODE | Should -Be 0
    }

    It 'STDOUT captured via STDERR when calling resource directly' {
        $input_json = @"
        {
            "executable": "pwsh",
            "arguments": ["-Command", "echo hello world"]
        }
"@
        $input_json | runcommandonset set 2> $TestDrive/output.txt
        $actual = Get-Content -Path $TestDrive/output.txt
        $actual | Should -Contain 'Stdout: hello'
        $actual | Should -Contain 'world'
        $LASTEXITCODE | Should -Be 0
    }

    It 'STDERR captured when calling resource directly with invalid args' {
        $json = runcommandonset set -e pwsh -a "echo hello world" 2> $TestDrive/output.txt
        $stdout = $json | ConvertFrom-Json
        $stdout.exit_code | Should -Be 64
        $expected = "Stderr: The argument 'echo hello world' is not recognized as the name of a script file. Check the spelling of the name, or if a path was included, verify that the path is correct and try again."
        $stderr = Get-Content -Path $TestDrive/output.txt
        $stderr | Should -Contain $expected
        $LASTEXITCODE | Should -Be 0
    }

    It 'Executable is a required input via CLI arguments' {
        $null = runcommandonset set -a foo
        $LASTEXITCODE | Should -Be 4
    }

    It 'Executable is a required input via STDIN' {
        $null = '{ "arguments": "foo" }' | dsc resource set -r Microsoft.DSC.Transitional/RunCommandOnSet
        $LASTEXITCODE | Should -Be 2
    }

    It 'Executable can be provided without arguments' {
        $result = '{ "executable": "pwsh" }' | dsc resource set -r Microsoft.DSC.Transitional/RunCommandOnSet | ConvertFrom-Json
        $result.changedProperties | Should -Be @()
        $LASTEXITCODE | Should -Be 0
    }

    It 'Exit code does not need to be provided to detect difference' {
        $result = '{ "executable": "pwsh", "arguments": ["invalid input"] }' | dsc resource set -r Microsoft.DSC.Transitional/RunCommandOnSet | ConvertFrom-Json
        $result.changedProperties | Should -Be @( 'exit_code' )
        $LASTEXITCODE | Should -Be 0
    }

    It 'Executable does not exist' {
        '{ "executable": "foo" }' | dsc resource set -r Microsoft.DSC.Transitional/RunCommandOnSet 2> $TestDrive/output.txt
        $actual = Get-Content -Path $TestDrive/output.txt
        $expected_logging = 'Failed to execute foo: No such file or directory (os error 2)'
        if ($IsWindows) {
            $expected_logging = 'Failed to execute foo: program not found'
        }
        $found_logging = $false
        ForEach ($line in $actual) {
            try {
                $log = $line | ConvertFrom-Json
                if ($log.fields.message -eq $expected_logging) {
                    $found_logging = $true
                    break
                }
            } catch {
                # skip lines that aren't JSON
            }
        }
        $found_logging | Should -Be $true
        $LASTEXITCODE | Should -Be 2
    }
}
