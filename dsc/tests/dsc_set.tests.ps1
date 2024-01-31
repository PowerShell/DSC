# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config set tests' {
    BeforeEach {
        if ($IsWindows) {
            $json = @'
            {
                "keyPath": "HKCU\\1\\2\\3",
                "_exist": false
            }
'@
            $json | registry config set
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
            $json | registry config set
        }
    }

    It 'can set and remove a registry value' -Skip:(!$IsWindows) {
        $json = @'
        {
            "keyPath": "HKCU\\1\\2\\3",
            "valueName": "Hello",
            "valueData": {
                "String": "World"
            }
        }
'@
        $out = $json | dsc resource set -r Microsoft.Windows/registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.afterState.keyPath | Should -Be 'HKCU\1\2\3'
        $result.afterState.valueName | Should -Be 'Hello'
        $result.afterState.valueData.String | Should -Be 'World'
        $result.changedProperties | Should -Be @('valueName', 'valueData', '_exist')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3

        $out = $json | dsc resource get -r Microsoft.Windows/registry
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
        $out = $json | dsc resource set -r Microsoft.Windows/registry
        $LASTEXITCODE | Should -Be 0
        $result = $out | ConvertFrom-Json
        $result.afterState.keyPath | Should -BeExactly 'HKCU\1'
        $result.changedProperties | Should -Be @('_exist')
        ($result.psobject.properties | Measure-Object).Count | Should -Be 3
    }

    It 'can accept the use of --format <format> as a subcommand' -Skip:(!$IsWindows) -TestCases @(
        @{ format = 'yaml'; expected = @'
beforeState:
  $id: https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json
  keyPath: HKCU\1
  _exist: false
afterState:
  $id: https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json
  keyPath: HKCU\1
  _exist: false
changedProperties: []
'@ }
        @{ format = 'json'; expected = '{"beforeState":{"$id":"https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json","keyPath":"HKCU\\1","_exist":false},"afterState":{"$id":"https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json","keyPath":"HKCU\\1","_exist":false},"changedProperties":[]}' }
        @{ format = 'pretty-json'; expected = @'
{
  "beforeState": {
    "$id": "https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json",
    "keyPath": "HKCU\\1",
    "_exist": false
  },
  "afterState": {
    "$id": "https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json",
    "keyPath": "HKCU\\1",
    "_exist": false
  },
  "changedProperties": []
}
'@ }
    ) {
        param($format, $expected)

        $json = @'
        {
            "keyPath": "HKCU\\1",
            "_exist": false
        }
'@

        $out =  $json | dsc resource set -r Microsoft.Windows/registry --format $format | Out-String
        $LASTEXITCODE | Should -Be 0
        $out.Trim() | Should -BeExactly $expected
    }

    It 'set can be used on a resource that does not implement test' {
        $manifest = @'
        {
            "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json",
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
        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $TestDrive
            $out = '{ "test": true }' | dsc resource set -r Test/SetNoTest | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.BeforeState.test | Should -Be $true
            $out.AfterState.test | Should -Be $false
        }
        finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }
}
