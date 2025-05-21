# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for resource discovery' {
    BeforeAll {
        $env:DSC_RESOURCE_PATH = $testdrive

        $script:lookupTableFilePath = if ($IsWindows) {
            Join-Path $env:LocalAppData "dsc\AdaptedResourcesLookupTable.json"
        } else {
            Join-Path $env:HOME ".dsc" "AdaptedResourcesLookupTable.json"
        }
    }

    AfterEach {
        Remove-Item -Path "$testdrive/test.dsc.resource.*" -ErrorAction SilentlyContinue
    }

    AfterAll {
        $env:DSC_RESOURCE_PATH = $null
    }

    It 'Use DSC_RESOURCE_PATH instead of PATH when defined' {
        $resourceJson = @'
        {
            "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
            "type": "DSC/TestPathResource",
            "version": "0.1.0",
            "get": {
              "executable": "dsc"
            }
          }
'@

        Set-Content -Path "$testdrive/test.dsc.resource.json" -Value $resourceJson
        $resources = dsc resource list | ConvertFrom-Json
        $resources.Count | Should -Be 1
        $resources.type | Should -BeExactly 'DSC/TestPathResource'
    }

    It 'support discovering <extension>' -TestCases @(
        @{ extension = 'yaml' }
        @{ extension = 'yml' }
    ) {
        param($extension)

        $resourceYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
        type: DSC/TestYamlResource
        version: 0.1.0
        get:
          executable: dsc
'@

        Set-Content -Path "$testdrive/test.dsc.resource.$extension" -Value $resourceYaml
        $resources = dsc resource list | ConvertFrom-Json
        $resources.Count | Should -Be 1
        $resources.type | Should -BeExactly 'DSC/TestYamlResource'
    }

    It 'does not support discovering a file with an extension that is not json or yaml' {
        param($extension)

        $resourceInput = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
        type: DSC/TestYamlResource
        version: 0.1.0
        get:
          executable: dsc
'@

        Set-Content -Path "$testdrive/test.dsc.resource.txt" -Value $resourceInput
        $resources = dsc resource list | ConvertFrom-Json
        $resources.Count | Should -Be 0
    }

    It 'warns on invalid semver' {
        $manifest = @'
        {
            "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
            "type": "Test/InvalidSemver",
            "version": "1.1.0..1",
            "get": {
                "executable": "dsctest"
            },
            "schema": {
                "command": {
                    "executable": "dsctest"
                }
            }
        }
'@
        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $testdrive
            Set-Content -Path "$testdrive/test.dsc.resource.json" -Value $manifest
            $out = dsc resource list 2>&1
            write-verbose -verbose ($out | Out-String)
            $out | Should -Match 'WARN.*?Validation.*?invalid version' -Because ($out | Out-String)
        }
        finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }

    It 'Ensure List operation populates adapter lookup table' {
        # remove adapter lookup table file
        Remove-Item -Force -Path $script:lookupTableFilePath -ErrorAction SilentlyContinue
        Test-Path $script:lookupTableFilePath -PathType Leaf | Should -BeFalse

        # perform List on an adapter - this should create adapter lookup table file
        $oldPSModulePath = $env:PSModulePath
        $TestClassResourcePath = Resolve-Path "$PSScriptRoot/../../powershell-adapter/Tests"
        $env:DSC_RESOURCE_PATH = $null
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $TestClassResourcePath
        dsc resource list -a Microsoft.DSC/PowerShell | Out-Null
        gc -raw $script:lookupTableFilePath
        $script:lookupTableFilePath | Should -FileContentMatchExactly 'Microsoft.DSC/PowerShell'
        Test-Path $script:lookupTableFilePath -PathType Leaf | Should -BeTrue
        $env:PSModulePath = $oldPSModulePath
    }

    It 'Ensure non-List operation populates adapter lookup table' {

        # remove adapter lookup table file
        Remove-Item -Force -Path $script:lookupTableFilePath -ErrorAction SilentlyContinue
        Test-Path $script:lookupTableFilePath -PathType Leaf | Should -BeFalse

        # perform Get on an adapter - this should create adapter lookup table file
        $oldPSModulePath = $env:PSModulePath
        $TestClassResourcePath = Resolve-Path "$PSScriptRoot/../../powershell-adapter/Tests"
        $env:DSC_RESOURCE_PATH = $null
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $TestClassResourcePath
        "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource' -f - | Out-Null

        Test-Path $script:lookupTableFilePath -PathType Leaf | Should -BeTrue
        $script:lookupTableFilePath | Should -FileContentMatchExactly 'testclassresource/testclassresource'
        $env:PSModulePath = $oldPSModulePath
    }

    It 'Verify adapter lookup table is used on repeat invocations' {

        $oldPSModulePath = $env:PSModulePath
        $TestClassResourcePath = Resolve-Path "$PSScriptRoot/../../powershell-adapter/Tests"
        $env:DSC_RESOURCE_PATH = $null
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $TestClassResourcePath

        # remove adapter lookup table file
        Remove-Item -Force -Path $script:lookupTableFilePath -ErrorAction Stop
        Test-Path $script:lookupTableFilePath -PathType Leaf | Should -BeFalse

        # initial invocation should populate and save adapter lookup table
        dsc -l trace resource list -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Read 0 items into lookup table"
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Saving lookup table"

        # second invocation (without an update) should use but not save adapter lookup table
        "{'Name':'TestClassResource1'}" | dsc -l trace resource get -r 'TestClassResource/TestClassResource' -f - 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Lookup table found resource 'testclassresource/testclassresource' in adapter 'Microsoft.DSC/PowerShell'"
        "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly "Saving lookup table"

        # third invocation (with an update) should save updated adapter lookup table
        dsc -l trace resource list -a Test/TestGroup 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly "Saving lookup table"

        $env:PSModulePath = $oldPSModulePath
    }

    It 'Verify non-zero exit code when resource not found' {

        $out = dsc resource get -r abc/def
        $LASTEXITCODE | Should -Be 7
        $out = dsc resource get --all -r abc/def
        $LASTEXITCODE | Should -Be 7
        $out = 'abc' | dsc resource set -r abc/def -f -
        $LASTEXITCODE | Should -Be 7
        $out = 'abc' | dsc resource test -r abc/def -f -
        $LASTEXITCODE | Should -Be 7
        $out = 'abc' | dsc resource delete -r abc/def -f -
        $LASTEXITCODE | Should -Be 7
        $out = dsc resource export -r abc/def
        $LASTEXITCODE | Should -Be 7
        $out = dsc resource schema -r abc/def
        $LASTEXITCODE | Should -Be 7
    }

    It 'Verify warning message when executable not found for: <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
        @{ operation = 'delete' }
        @{ operation = 'export' }
        @{ operation = 'resolve' }
        @{ operation = 'whatIf' }
    ) {
        param($operation)

        $manifest = @"
        {
            "`$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
            "type": "Test/ExecutableNotFound",
            "version": "0.1.0",
            "$operation": {
                "executable": "doesNotExist"
            }
        }
"@
        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $testdrive
            Set-Content -Path "$testdrive/test.dsc.resource.json" -Value $manifest
            $out = dsc resource list 'Test/ExecutableNotFound' 2> "$testdrive/error.txt" | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.Count | Should -Be 1
            $out.Type | Should -BeExactly 'Test/ExecutableNotFound'
            $out.Kind | Should -BeExactly 'resource'
            Get-Content -Path "$testdrive/error.txt" | Should -Match "WARN.*?Executable 'doesNotExist' not found"
        }
        finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }

    It 'DSC_RESOURCE_PATH should be used for executable lookup' {
        $dscTest = Get-Command dscecho -ErrorAction Stop
        $target = if ($IsWindows) {
            'echoIt.exe'
        } else {
            'echoIt'
        }
        Copy-Item -Path "$($dscTest.Source)" -Destination "$testdrive\$target"
        $manifest = Get-Content -Raw -Path "$(Split-Path -Path $dscTest.Source -Parent)\echo.dsc.resource.json" | ConvertFrom-Json
        $manifest.type = 'Test/MyEcho'
        $manifest.get.executable = $target
        $manifest.set = $null
        $manifest.test = $null
        $manifest.schema.command.executable = $target
        Set-Content -Path "$testdrive/test.dsc.resource.json" -Value ($manifest | ConvertTo-Json -Depth 10)

        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $testdrive
            $out = dsc resource get -r 'Test/MyEcho' -i '{"output":"Custom"}' 2> "$testdrive/error.txt" | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.actualState.output | Should -BeExactly 'Custom'
        }
        finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }
}
