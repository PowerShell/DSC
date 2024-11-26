# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter resource tests' {

    BeforeAll {
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot

        if ($IsLinux -or $IsMacOS) {
            $cacheFilePath = Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
        }
        else
        {
            $cacheFilePath = Join-Path $env:LocalAppData "dsc" "PSAdapterCache.json"
        }
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    BeforeEach {
        Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath
    }

    It 'Discovery includes class-based resources' {

        $r = dsc resource list '*' -a Microsoft.DSC/PowerShell
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | ? {$_.Type -eq 'TestClassResource/TestClassResource'}).Count | Should -Be 1
    }

    It 'Get works on class-based resource' {

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.Prop1 | Should -BeExactly 'ValueForProp1'

        # verify that only properties with DscProperty attribute are returned
        $propertiesNames = $res.actualState.result.properties | Get-Member -MemberType NoteProperty | % Name
        $propertiesNames | Should -Not -Contain 'NonDscProperty'
        $propertiesNames | Should -Not -Contain 'HiddenNonDscProperty'
    }

    It 'Get uses enum names on class-based resource' {

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.EnumProp | Should -BeExactly 'Expected'
    }

    It 'Test works on class-based resource' {

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource test -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.InDesiredState | Should -Be $True
        $res.actualState.result.properties.InDesiredState.GetType().Name | Should -Be "Boolean"

        # verify that only properties with DscProperty attribute are returned
        $propertiesNames = $res.actualState.result.properties.InDesiredState | Get-Member -MemberType NoteProperty | % Name
        $propertiesNames | Should -Not -Contain 'NonDscProperty'
        $propertiesNames | Should -Not -Contain 'HiddenNonDscProperty'
    }

    It 'Set works on class-based resource' {

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource set -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.result | Should -Not -BeNull
    }

    It 'Export works on PS class-based resource' {

        $r = dsc resource export -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.resources[0].properties.result.count | Should -Be 5
        $res.resources[0].properties.result[0].Name | Should -Be "Object1"
        $res.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"

        # verify that only properties with DscProperty attribute are returned
        $res.resources[0].properties.result | %{
            $propertiesNames = $_ | Get-Member -MemberType NoteProperty | % Name
            $propertiesNames | Should -Not -Contain 'NonDscProperty'
            $propertiesNames | Should -Not -Contain 'HiddenNonDscProperty'
        }
    }

    It 'Get --all works on PS class-based resource' {

        $r = dsc resource get --all -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.count | Should -Be 5
        $res.actualState.result| % {$_.Name | Should -Not -BeNullOrEmpty}
    }

    It 'Verify that ClearCache works in PSAdapter' {
        # generate the cache
        $null = dsc resource list '*' -a Microsoft.DSC/PowerShell
        # call the ClearCache operation
        $scriptPath = Join-Path $PSScriptRoot '..' 'psDscAdapter' 'powershell.resource.ps1'
        $null = & $scriptPath -Operation ClearCache
        # verify that PSAdapter does not find the cache
        dsc -l debug resource list '*' -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 0
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Cache file not found'
    }

    It 'Verify that a new PS Cache version results in cache rebuid' {
        # generate the cache
        $null = dsc resource list '*' -a Microsoft.DSC/PowerShell
        # update the version in the cache file
        $cacheFilePath = if ($IsWindows) {
            # PS 6+ on Windows
            Join-Path $env:LocalAppData "dsc\PSAdapterCache.json"
        } else {
            # either WinPS or PS 6+ on Linux/Mac
            if ($PSVersionTable.PSVersion.Major -le 5) {
                Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"
            } else {
                Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
            }
        }
        $cache = Get-Content -Raw $cacheFilePath | ConvertFrom-Json
        $cache.CacheSchemaVersion = 0
        $jsonCache = $cache | ConvertTo-Json -Depth 90
        New-Item -Force -Path $cacheFilePath -Value $jsonCache -Type File | Out-Null

        # verify that a new PS Cache version results in cache rebuid
        dsc -l debug resource list '*' -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        $LASTEXITCODE | Should -Be 0
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Incompatible version of cache in file'
    }

    It 'Verify that removing a module results in cache rebuid' {

        Copy-Item -Recurse -Force -Path "$PSScriptRoot/TestClassResource" -Destination $TestDrive
        Copy-Item -Recurse -Force -Path "$PSScriptRoot/TestClassResource" -Destination "$PSScriptRoot/Backup/TestClassResource"
        Remove-Item -Recurse -Force -Path "$PSScriptRoot/TestClassResource"

        $oldPath = $env:PSModulePath
        try {
            $env:PSModulePath += [System.IO.Path]::PathSeparator + $TestDrive

            # generate the cache
            $null = dsc resource list '*' -a Microsoft.DSC/PowerShell
            # remove the module files
            Remove-Item -Recurse -Force -Path "$TestDrive/TestClassResource"
            # verify that cache rebuid happened
            dsc -l trace resource list '*' -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt

            $LASTEXITCODE | Should -Be 0
            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Detected non-existent cache entry'
            "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Constructing Get-DscResource cache'
        }
        finally {
            $env:PSModulePath = $oldPath
            Copy-Item -Recurse -Force -Path "$PSScriptRoot/Backup/TestClassResource" -Destination "$PSScriptRoot"
            Remove-Item -Recurse -Force -Path "$PSScriptRoot/Backup"
        }
    }

    It 'Verify inheritance works in class-based resources' {

        $r = dsc resource list '*' -a Microsoft.DSC/PowerShell
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        $t = $resources | ? {$_.Type -eq 'TestClassResource/TestClassResource'}
        $t.properties | Should -Contain "BaseProperty"
    }

    It 'Verify highest module version is loaded' {

        $srcPath = Join-Path $PSScriptRoot 'TestClassResource'
        $pathRoot1 = Join-Path $TestDrive 'A'
        $pathRoot2 = Join-Path $TestDrive 'B'
        $path1 = Join-Path $pathRoot1 'TestClassResource' '1.0'
        $path2 = Join-Path $pathRoot1 'TestClassResource' '1.1'
        $path3 = Join-Path $pathRoot2 'TestClassResource' '2.0'
        $path4 = Join-Path $pathRoot2 'TestClassResource' '2.0.1'

        New-Item -ItemType Directory -Force -Path $path1 | Out-Null
        New-Item -ItemType Directory -Force -Path $path2 | Out-Null
        New-Item -ItemType Directory -Force -Path $path3 | Out-Null
        New-Item -ItemType Directory -Force -Path $path4 | Out-Null

        $files = Get-ChildItem -Recurse -File -Path $srcPath
        $files | Copy-Item -Destination $path1
        $files | Copy-Item -Destination $path2
        $files | Copy-Item -Destination $path3
        $files | Copy-Item -Destination $path4

        $filePath = Join-Path $path1 'TestClassResource.psd1'
        (Get-Content -Raw $filePath).Replace("ModuleVersion = `'0.0.1`'", "ModuleVersion = `'1.0`'") | Set-Content $filePath
        $filePath = Join-Path $path2 'TestClassResource.psd1'
        (Get-Content -Raw $filePath).Replace("ModuleVersion = `'0.0.1`'", "ModuleVersion = `'1.1`'") | Set-Content $filePath
        $filePath = Join-Path $path3 'TestClassResource.psd1'
        (Get-Content -Raw $filePath).Replace("ModuleVersion = `'0.0.1`'", "ModuleVersion = `'2.0`'") | Set-Content $filePath
        $filePath = Join-Path $path4 'TestClassResource.psd1'
        (Get-Content -Raw $filePath).Replace("ModuleVersion = `'0.0.1`'", "ModuleVersion = `'2.0.1`'") | Set-Content $filePath


        $oldPath = $env:PSModulePath
        try {
            $env:PSModulePath += [System.IO.Path]::PathSeparator + $pathRoot1
            $env:PSModulePath += [System.IO.Path]::PathSeparator + $pathRoot2

            $r = dsc resource list '*' -a Microsoft.DSC/PowerShell
            $LASTEXITCODE | Should -Be 0
            $resources = $r | ConvertFrom-Json
            $r = @($resources | ? {$_.Type -eq 'TestClassResource/TestClassResource'})
            $r.Count | Should -Be 1
            $r[0].Version | Should -Be '2.0.1'
        }
        finally {
            $env:PSModulePath = $oldPath
        }
    }

    It 'Verify adapted_dsc_type field in Get' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = '{TestCaseId: 1}'| dsc resource get -r 'Test/TestCase' -f -
            $LASTEXITCODE | Should -Be 0
            $resources = $r | ConvertFrom-Json
            $resources.actualState.result | Should -Be $True
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Verify adapted_dsc_type field in Set' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = '{TestCaseId: 1}'| dsc resource set -r 'Test/TestCase' -f -
            $LASTEXITCODE | Should -Be 0
            $resources = $r | ConvertFrom-Json
            $resources.beforeState.result | Should -Be $True
            $resources.afterState.result | Should -Be $True
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Verify adapted_dsc_type field in Test' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = '{TestCaseId: 1}'| dsc resource test -r 'Test/TestCase' -f -
            $LASTEXITCODE | Should -Be 0
            $resources = $r | ConvertFrom-Json
            $resources.actualState.result | Should -Be $True
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Verify adapted_dsc_type field in Export' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = dsc resource export -r 'Test/TestCase'
            $LASTEXITCODE | Should -Be 0
            $resources = $r | ConvertFrom-Json
            $resources.resources[0].properties.result | Should -Be $True
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Dsc can process large resource output' {
        $env:TestClassResourceResultCount = 5000 # with sync resource invocations this was not possible

        dsc resource list -a Microsoft.DSC/PowerShell | Out-Null
        $r = dsc resource export -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.resources[0].properties.result.count | Should -Be 5000

        $env:TestClassResourceResultCount = $null
    }

    It 'Verify that there are no cache rebuilds for several sequential executions' {

        # remove cache file
        $cacheFilePath = if ($IsWindows) {
            # PS 6+ on Windows
            Join-Path $env:LocalAppData "dsc\PSAdapterCache.json"
        } else {
            # either WinPS or PS 6+ on Linux/Mac
            if ($PSVersionTable.PSVersion.Major -le 5) {
                Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"
            } else {
                Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
            }
        }
        Remove-Item -Force -Path $cacheFilePath -ErrorAction Ignore

        # first execution should build the cache
        dsc -l trace resource list -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Constructing Get-DscResource cache'

        # next executions following shortly after should Not rebuild the cache
        1..3 | %{
            dsc -l trace resource list -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Constructing Get-DscResource cache'
        }
    }
}
