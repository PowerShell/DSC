# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config argument tests' {
    BeforeAll {
        $manifest = @'
        {
            "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
            "type": "Test/Hello",
            "version": "0.1.0",
            "get": {
                "executable": "pwsh",
                "args": [
                    "-NoLogo",
                    "-NonInteractive",
                    "-NoProfile",
                    "-Command",
                    "'{ \"hello\": \"world\" }'"
                ]
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
                        "hello": {
                            "type": "string",
                            "description": "test"
                        }
                    }
                }
            }
        }
'@

        Set-Content -Path "$TestDrive/Hello.dsc.resource.json" -Value $manifest
        $oldPath = $env:DSC_RESOURCE_PATH
        $sep = [System.IO.Path]::PathSeparator
        $env:DSC_RESOURCE_PATH = $env:PATH + $sep + $TestDrive
    }

    AfterEach {
        if (Test-Path $TestDrive/error.txt) {
            Remove-Item -Path $TestDrive/error.txt
        }
    }

    AfterAll {
        $env:DSC_RESOURCE_PATH = $oldPath
    }

    It 'input is <type>' -TestCases @(
        @{ type = 'yaml'; text = @'
            output: Hello There
'@ }
        @{ type = 'json'; text = @'
            {
                "output": "Hello There"
            }
'@ }
    ) {
        param($text)
        $output = $text | dsc resource get -r Microsoft.DSC.Debug/Echo -f -
        $output = $output | ConvertFrom-Json
        $output.actualState.output | Should -BeExactly 'Hello There'
    }

    It '--output-format <format> is used even when redirected' -TestCases @(
        @{ format = 'yaml'; expected = @'
actualState:
  hello: world
'@ }
        @{ format = 'json'; expected = '{"actualState":{"hello":"world"}}' }
        @{ format = 'pretty-json'; expected = @'
{
  "actualState": {
    "hello": "world"
  }
}
'@ }
    ) {
        param($format, $expected)

        $out = dsc resource get -r Test/Hello --output-format $format | Out-String
        $LASTEXITCODE | Should -Be 0
        $out.Trim() | Should -BeExactly $expected
    }

    It 'YAML output includes object separator' {
        $out = dsc resource list -o yaml | Out-String
        foreach ($obj in $out.Split('---')) {
            $resource = $obj | y2j | ConvertFrom-Json
            $resource | Should -Not -BeNullOrEmpty
            $resource.Type | Should -BeLike '*/*'
            $resource.Kind | Should -BeIn ('resource', 'group', 'exporter', 'importer', 'adapter')
        }
    }

    It 'can generate PowerShell completer' {
        $out = dsc completer powershell | Out-String
        Invoke-Expression $out
        $completions = TabExpansion2 -inputScript 'dsc c'
        $completions.CompletionMatches.Count | Should -Be 2
        $completions.CompletionMatches[0].CompletionText | Should -Be 'completer'
        $completions.CompletionMatches[1].CompletionText | Should -Be 'config'
    }

    It 'input can be passed using <parameter>' -TestCases @(
        @{ parameter = '-i' }
        @{ parameter = '--input' }
    ) {
        param($parameter)

        $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: os
  type: Microsoft/OSInfo
  properties:
    family: Windows
'@

        $out = dsc config get $parameter "$yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].type | Should -BeExactly 'Microsoft/OSInfo'
    }

    It 'input can be passed using <parameter>' -TestCases @(
        @{ parameter = '-f' }
        @{ parameter = '--file' }
    ) {
        param($parameter)

        $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: os
  type: Microsoft/OSInfo
  properties:
    family: Windows
'@

        Set-Content -Path $TestDrive/foo.yaml -Value $yaml
        $out = dsc config get $parameter "$TestDrive/foo.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].type | Should -BeExactly 'Microsoft/OSInfo'
    }

    It '--input and --file cannot be used together' {
        dsc config get --input 1 --file foo.json 2> $TestDrive/error.txt
        $err = Get-Content $testdrive/error.txt -Raw
        $err.Length | Should -Not -Be 0
        $LASTEXITCODE | Should -Be 2
    }

    It '--trace-level has effect' {
        dsc -l debug resource get -r Microsoft/OSInfo 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'DEBUG'
        $LASTEXITCODE | Should -Be 0
    }

    It 'resource tracing shows up' -Skip:(!$IsWindows) {
        # Assumption here is that DSC/PowerShellGroup provider is visible
        dsc -l trace resource list * -a *PowerShell* 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'PSModulePath'
        $LASTEXITCODE | Should -Be 0
    }

    It 'stdin cannot be empty if neither input or path is provided' {
        '' | dsc resource set -r Microsoft/OSInfo -f - 2> $TestDrive/error.txt
        $err = Get-Content $testdrive/error.txt -Raw
        $err.Length | Should -Not -Be 0
        $LASTEXITCODE | Should -Be 4
    }

    It 'input cannot be empty if neither stdin or path is provided' {
        dsc resource set -r Microsoft/OSInfo --input " " 2> $TestDrive/error.txt
        $err = Get-Content $testdrive/error.txt -Raw
        $err.Length | Should -Not -Be 0
        $LASTEXITCODE | Should -Be 4
    }

    It 'path contents cannot be empty if neither stdin or input is provided' {
        Set-Content -Path $TestDrive/empty.yaml -Value " "
        dsc resource set -r Microsoft/OSInfo --file $TestDrive/empty.yaml 2> $TestDrive/error.txt
        $err = Get-Content $testdrive/error.txt -Raw
        $err.Length | Should -Not -Be 0
        $LASTEXITCODE | Should -Be 4
    }

    It 'document cannot be empty if neither stdin or path is provided' {
        dsc config set --input " " 2> $TestDrive/error.txt
        $err = Get-Content $testdrive/error.txt -Raw
        $err.Length | Should -Not -Be 0
        $LASTEXITCODE | Should -Be 4
    }

    It 'verify `dsc resource list` and `dsc resource list *`' {
        # return all native resources, providers, but not adapter-based resources;
        # results for `dsc resource list` and `dsc resource list *` should be the same
        $a = dsc resource list -o json
        $b = dsc resource list '*' -o json
        $a.Count | Should -Be $b.Count
        0..($a.Count-1) | %{
            $a_obj = $a[$_] | ConvertFrom-Json
            $b_obj = $b[$_] | ConvertFrom-Json
            $a_obj.type | Should -Be $b_obj.type
        }
    }

    It 'verify `dsc resource list resource_filter`' {
        # same as previous but also apply resource_filter filter
        $a = dsc resource list 'Test*' -o json
        0..($a.Count-1) | %{
            $a_obj = $a[$_] | ConvertFrom-Json
            $a_obj.type.StartsWith("Test") | Should -Be $true
            # adapter-based resources should Not be in the results
            $a_obj.requireAdapter | Should -BeNullOrEmpty
        }
    }

    It 'verify `dsc resource list * -a *`' {
        # return all adapter-based resources
        $a = dsc resource list '*' -a '*' -o json
        0..($a.Count-1) | %{
            $a_obj = $a[$_] | ConvertFrom-Json
            $a_obj.requireAdapter | Should -Not -BeNullOrEmpty
            $a_obj.kind | Should -Be "Resource"
        }
    }

    It 'verify `dsc resource list * adapter_filter`' {
        # return all resources of adapters that match adapter_filter filter
        $a = dsc resource list '*' -a Test* -o json | ConvertFrom-Json
        foreach ($r in $a) {
            $r.requireAdapter.StartsWith("Test") | Should -Be $true
            $r.kind | Should -Be "resource"
        }
    }

    It 'verify `dsc resource list resource_filter adapter_filter`' {
        # same as previous but also apply resource_filter filter to resource types
        $a = dsc resource list *TestResource2 -a *TestGroup -o json | ConvertFrom-Json
        $a.Count | Should -Be 1
        $r = $a[0]
        $r.requireAdapter | Should -Not -BeNullOrEmpty
        $r.requireAdapter.StartsWith("Test") | Should -Be $true
        $r.kind | Should -Be "resource"
    }

    It 'passing filepath to document arg should error' {
        $configFile = Resolve-Path $PSScriptRoot/../examples/osinfo.dsc.json
        $stderr = dsc config get -i $configFile 2>&1
        $stderr | Should -Match '.*?--file.*?'
    }

    It 'file containing UTF-8 BOM should be read correctly' {
        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: hello
'@
        Set-Content -Path "$TestDrive/utf8bom.yaml" -Value $yaml -Encoding utf8BOM
        $out = dsc config get --file "$TestDrive/utf8bom.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].type | Should -BeExactly 'Microsoft.DSC.Debug/Echo'
        $out.results[0].result.actualState.output | Should -BeExactly 'hello'
    }

    It 'Get operation on the adapter itself should fail' {
        dsc resource get -r Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Can not perform this operation on the adapter'
    }

    It 'Get-all operation on the adapter itself should fail' {
        dsc resource get --all -r Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Can not perform this operation on the adapter'
    }

    It 'Set operation on the adapter itself should fail' {
        'abc' | dsc resource set -r Microsoft.DSC/PowerShell -f - 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Can not perform this operation on the adapter'
    }

    It 'Test operation on the adapter itself should fail' {
        'abc' | dsc resource test -r Microsoft.DSC/PowerShell -f - 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Can not perform this operation on the adapter'
    }

    It 'Export operation on the adapter itself should fail' {
        dsc resource export -r Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Can not perform this operation on the adapter'
    }

    It 'Delete operation on the adapter itself should fail' {
        dsc resource delete -r Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 2
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Can not perform this operation on the adapter'
    }

    It 'Invalid --system-root' {
        dsc config --system-root /invalid/path get -f "$PSScriptRoot/../examples/groups.dsc.yaml" 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 1
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Target path does not exist: '/invalid/path'"
    }

    It '--progress-format can be None' {
        dsc -p none resource list 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 0
        (Get-Content $TestDrive/tracing.txt -Raw) | Should -BeNullOrEmpty
    }
}
