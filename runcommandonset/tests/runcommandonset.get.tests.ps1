# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for runcommandonset get' {
    BeforeAll {
        $oldPath = $env:DSC_RESOURCE_PATH
        $env:DSC_RESOURCE_PATH = Join-Path $PSScriptRoot ".."
    }

    AfterAll {
        $env:DSC_RESOURCE_PATH = $oldPath
    }

    It 'Input passed for executable, arguments, and exit code' {
        $json = @"
        {
            "executable": "foo",
            "arguments": ["bar", "baz"],
            "exitCode": 5,
        }
"@

        $result = $json | dsc resource get -r Microsoft.DSC.Transitional/RunCommandOnSet -f - | ConvertFrom-Json
        $result.actualState.arguments | Should -BeExactly @('bar', 'baz')
        $result.actualState.executable | Should -BeExactly 'foo'
        $result.actualState.exitCode | Should -BeExactly 5
    }

    It 'Executable is a required input via CLI arguments' {
        $null = runcommandonset get -a foo
        $LASTEXITCODE | Should -Be 4
    }

    It 'Executable is a required input via STDIN' {
        '{ "arguments": "foo" }' | dsc resource get -r Microsoft.DSC.Transitional/RunCommandOnSet -f -
        $LASTEXITCODE | Should -Be 2
    }

    It 'Input provided via configuration doc' {
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: get
              type: Microsoft.DSC.Transitional/RunCommandOnSet
              properties:
                executable: foo
                arguments:
                - "bar"
"@
        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.hadErrors | Should -BeFalse
        $out.results.Count | Should -Be 1
        $out.results[0].type | Should -BeExactly 'Microsoft.DSC.Transitional/RunCommandOnSet'
        $out.results[0].result.actualState.executable | Should -BeExactly 'foo'
        $out.results[0].result.actualState.arguments[0] | Should -BeExactly 'bar'
    }
}
