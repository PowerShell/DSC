# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Resource provider tests' {

    It 'Can list provider resources' {

        $out = dsc resource list *testresource* | ConvertFrom-Json | Sort-Object -Property type
        $out.Count | Should -Be 2
        $out[0].type | Should -BeExactly 'TestResource1'
        $out[0].version | Should -Be '1.0.0'
        $out[0].path | Should -BeExactly 'test_resource1'
        $out[0].implementedas | Should -BeExactly 'TestResource'
        $out[0].requires | Should -BeExactly 'Test/TestGroup'
        $out[1].type | Should -BeExactly 'TestResource2'
        $out[1].version | Should -Be '1.0.1'
        $out[1].path | Should -BeExactly 'test_resource2'
        $out[1].implementedas | Should -BeExactly 'TestResource'
        $out[1].requires | Should -BeExactly 'Test/TestGroup'
    }

    It 'Error if provider resource is missing "requires" member' {
        $invalid_manifest = @'
        {
            "manifestVersion": "1.0",
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
            $LASTEXITCODE | Should -Be 2
            ,$out | Should -Match ".*?'requires'*"
        }
        finally {
            $env:PATH = $oldPath
        }
    }
}
