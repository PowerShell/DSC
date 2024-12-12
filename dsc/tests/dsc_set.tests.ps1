# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'resource set tests' {
    BeforeAll {
        $manifest = @'
        {
            "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/resource/manifest.json",
            "type": "Test/SetNoTest",
            "version": "0.1.0",
            "get": {
                "executable": "pwsh",
                "args": [
                    "-NoLogo",
                    "-NonInteractive",
                    "-NoProfile",
                    "-Command",
                    "'{ \"test\": true }'"
                ]
            },
            "set": {
                "executable": "pwsh",
                "input": "stdin",
                "args": [
                    "-NoLogo",
                    "-NonInteractive",
                    "-NoProfile",
                    "-Command",
                    "'{ \"test\": false }'"
                ],
                "return": "state"
            },
            "schema": {
                "embedded": {
                    "$schema": "http://json-schema.org/draft-07/schema#",
                    "$id": "https://test",
                    "title": "test",
                    "description": "test",
                    "type": "object",
                    "required": [],
                    "additionalProperties": false,
                    "properties": {
                        "test": {
                            "type": "boolean",
                            "description": "test"
                        }
                    }
                }
            }
        }
'@

        Set-Content -Path "$TestDrive/SetNoTest.dsc.resource.json" -Value $manifest
    }

    BeforeEach {
        if ($IsWindows) {
            $json = @'
            {
                "keyPath": "HKCU\\1\\2\\3",
                "_exist": false
            }
'@
            $null = registry config set --input $json
        }
    }

    AfterEach {
        if ($IsWindows) {
            $json = @'
            {
                "keyPath": "HKCU\\1",
                "_exist": false
            }
'@
            $null = registry config set --input $json
        }
    }

    # test pending changes in engine to call delete if _exist is not handled directly
    It 'can set and remove a registry value' -Pending {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $out = $json | dsc resource set -r Microsoft.Windows/Registry -f -
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.afterState.keyPath | Should -Be 'HKCU\1\2\3'
        $result.afterState.valueName | Should -Be 'Hello'
        $result.afterState.valueData.String | Should -Be 'World'
        $result.changedProperties | Should -Be @('valueName', 'valueData', '_exist')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3

        $out = $json | dsc resource get -r Microsoft.Windows/Registry -f -
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.actualState.keyPath | Should -Be 'HKCU\1\2\3'
        $result.actualState.valueName | Should -Be 'Hello'
        $result.actualState.valueData.String | Should -Be 'World'
        ($result.psobject.properties | Measure-Object).Count | Should -Be 1

        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_exist": false
        }
'@
        $out = $json | dsc resource set -r Microsoft.Windows/Registry -f -
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.afterState.keyPath | Should -BeExactly 'HKCU\1'
        $result.changedProperties | Should -Be @('_exist')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }

    It 'can accept the use of --output-format <format> as a subcommand' -Skip:(!$IsWindows) -TestCases @(
        @{ format = 'yaml'; expected = @'
beforeState:
  test: true
afterState:
  test: false
changedProperties:
- test
'@ }
        @{ format = 'json'; expected = '{"beforeState":{"test":true},"afterState":{"test":false},"changedProperties":["test"]}' }
        @{ format = 'pretty-json'; expected = @'
{
  "beforeState": {
    "test": true
  },
  "afterState": {
    "test": false
  },
  "changedProperties": [
    "test"
  ]
}
'@ }
    ) {
        param($format, $expected)

        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $TestDrive
            $out = '{ "test": true }' | dsc resource set -r Test/SetNoTest -f - --output-format $format | Out-String
            $LASTEXITCODE | Should -Be 0
            $out.Trim() | Should -BeExactly $expected
        }
        finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }

    It 'set can be used on a resource that does not implement test' {
        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $TestDrive
            $out = '{ "test": true }' | dsc resource set -r Test/SetNoTest -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.BeforeState.test | Should -Be $true
            $out.AfterState.test | Should -Be $false
        }
        finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }
}
