# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Resource provider tests' {

    It 'Can list provider resources' {

        $out = dsc resource list *testresource* | ConvertFrom-Json | Sort-Object -Property type
        $out.Count | Should -Be 2
        $out[0].type | Should -BeExactly 'Test/TestResource1'
        $out[0].version | Should -Be '1.0.0'
        $out[0].path | Should -BeExactly 'test_resource1'
        $out[0].implementedas | Should -BeExactly 'TestResource'
        $out[0].requires | Should -BeExactly 'Test/TestGroup'
        $out[1].type | Should -BeExactly 'Test/TestResource2'
        $out[1].version | Should -Be '1.0.1'
        $out[1].path | Should -BeExactly 'test_resource2'
        $out[1].implementedas | Should -BeExactly 'TestResource'
        $out[1].requires | Should -BeExactly 'Test/TestGroup'
    }

    It 'Error if provider resource is missing "requires" member' {
        $invalid_manifest = @'
        {
            "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json",
            "type": "Test/InvalidTestGroup",
            "version": "0.1.0",
            "get": {
                "executable": "test_group_resource",
                "args": [
                    "get"
                ]
            },
            "schema": {
                "command": {
                    "executable": "test_group_resource",
                    "args": [
                        "schema"
                    ]
                }
            },
            "provider": {
                "list": {
                    "executable": "test_group_resource",
                    "args": [
                        "listmissingrequires"
                    ]
                },
                "config": "sequence"
            }
        }
'@
        $oldPath = $env:PATH
        try {
            Set-Content -Path testdrive:/invalid.dsc.resource.json -Value $invalid_manifest
            $env:PATH += [System.IO.Path]::PathSeparator + (Resolve-Path (Resolve-Path $TestDrive -Relative))

            $out = dsc resource list *invalid* 2>&1
            $LASTEXITCODE | Should -Be 0
            ,$out | Should -Match ".*?'requires'*"
        }
        finally {
            $env:PATH = $oldPath
        }
    }
}
