# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter resource tests' {

    BeforeAll {
        $OldPSModulePath = $env:PSModulePath
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot

        if ($IsLinux -or $IsMacOS) {
            $cacheFilePath = Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
        }
        else {
            $cacheFilePath = Join-Path $env:LocalAppData "dsc" "PSAdapterCache.json"
        }
    }

    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    BeforeEach {
        Remove-Item -Force -ErrorAction Ignore -Path $cacheFilePath
    }

    It 'Discovery includes class-based resources' {

        $r = dsc resource list '*' -a Microsoft.DSC/PowerShell
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | Where-Object { $_.Type -eq 'TestClassResource/TestClassResource' }).Count | Should -Be 1
        ($resources | Where-Object -Property type -EQ 'TestClassResource/TestClassResource').capabilities | Should -BeIn @('get', 'set', 'test', 'export')
        ($resources | Where-Object -Property type -EQ 'TestClassResource/NoExport').capabilities | Should -BeIn @('get', 'set', 'test')
    }

    It 'Get works on class-based resource' {

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.Prop1 | Should -BeExactly 'ValueForProp1'

        # verify that only properties with DscProperty attribute are returned
        $propertiesNames = $res.actualState | Get-Member -MemberType NoteProperty | % Name
        $propertiesNames | Should -Not -Contain 'NonDscProperty'
        $propertiesNames | Should -Not -Contain 'HiddenNonDscProperty'
    }

    It 'Get uses enum names on class-based resource' {

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.EnumProp | Should -BeExactly 'Expected'
    }

    It 'Get should return the correct properties on class-based resource' {
        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json -AsHashtable
        $res.actualState.ContainsKey('Name') | Should -Be $True
        $res.actualState.ContainsKey('Prop1') | Should -Be $True
        $res.actualState.ContainsKey('HashTableProp') | Should -Be $True
        $res.actualState.ContainsKey('EnumProp') | Should -Be $True
        $res.actualState.ContainsKey('Credential') | Should -Be $True
        $res.actualState.ContainsKey('Ensure') | Should -Be $True
        $res.actualState.ContainsKey('BaseProperty') | Should -Be $True
        $res.actualState.ContainsKey('HiddenDscProperty') | Should -Be $True
        $res.actualState.ContainsKey('NonDscProperty') | Should -Be $False
        $res.actualState.ContainsKey('HiddenNonDscProperty') | Should -Be $False
    }

    It 'Test works on class-based resource' {

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource test -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.InDesiredState | Should -Be $True -Because $r
    }

    It 'Set works on class-based resource' {

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource set -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.Prop1 | Should -BeExactly 'ValueForProp1'
        $res.changedProperties | Should -BeNullOrEmpty
    }

    It 'Export works on PS class-based resource' -Pending {

        $r = dsc resource export -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.resources[0].properties.result.count | Should -Be 5
        $res.resources[0].properties.result[0].Name | Should -Be "Object1"
        $res.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"

        # verify that only properties with DscProperty attribute are returned
        $res.resources[0].properties.result | % {
            $propertiesNames = $_ | Get-Member -MemberType NoteProperty | % Name
            $propertiesNames | Should -Not -Contain 'NonDscProperty'
            $propertiesNames | Should -Not -Contain 'HiddenNonDscProperty'
        }
    }

    It 'Get --all works on PS class-based resource' -Pending {

        $r = dsc resource get --all -r TestClassResource/TestClassResource 2>$null
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.count | Should -Be 5
        $res.actualState.result | % { $_.Name | Should -Not -BeNullOrEmpty }
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
        }
        else {
            # either WinPS or PS 6+ on Linux/Mac
            if ($PSVersionTable.PSVersion.Major -le 5) {
                Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"
            }
            else {
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
            $null = dsc -l trace resource list '*' -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt

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
        $t = $resources | ? { $_.Type -eq 'TestClassResource/TestClassResource' }
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
            $r = @($resources | ? { $_.Type -eq 'TestClassResource/TestClassResource' })
            $r.Count | Should -Be 1
            $r[0].Version | Should -Be '2.0.1'
        }
        finally {
            $env:PSModulePath = $oldPath
        }
    }

    It 'Verify invoke Get on adapted resource' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = '{"TestCaseId": 1}' | dsc -l trace resource get -r 'Test/TestCase' -f - 2> $TestDrive/tracing.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/tracing.txt | Out-String)
            $resources = $r | ConvertFrom-Json
            $resources.actualState.TestCaseid | Should -Be 1
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Verify invoke Set on adapted resource' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = '{"TestCaseId": 1}' | dsc resource set -r 'Test/TestCase' -f - 2> $TestDrive/tracing.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/tracing.txt | Out-String)
            $resources = $r | ConvertFrom-Json
            $resources.beforeState.TestCaseid | Should -Be 1
            $resources.afterState.TestCaseId | Should -Be 1
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Verify invoke Test on adapted resource' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = '{"TestCaseId": 1}' | dsc resource test -r 'Test/TestCase' -f -
            $LASTEXITCODE | Should -Be 0
            $resources = $r | ConvertFrom-Json
            $resources.actualState.TestCaseId | Should -Be 1
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Verify invoke Export on adapted resource' {
        $oldPath = $env:PATH
        try {
            $adapterPath = Join-Path $PSScriptRoot 'TestAdapter'
            $env:PATH += [System.IO.Path]::PathSeparator + $adapterPath

            $r = dsc -l trace resource export -r 'Test/TestCase' 2> $TestDrive/tracing.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/tracing.txt | Out-String)
            $resources = $r | ConvertFrom-Json
            $resources.resources.count | Should -Be 2
            $resources.resources[0].type | Should -BeExactly 'Test/TestCase'
            $resources.resources[0].name | Should -BeExactly 'TestCase-0'
            $resources.resources[0].properties.TestCaseId | Should -Be 1
            $resources.resources[1].type | Should -BeExactly 'Test/TestCase'
            $resources.resources[1].name | Should -BeExactly 'TestCase-1'
            $resources.resources[1].properties.TestCaseId | Should -Be 2
        }
        finally {
            $env:PATH = $oldPath
        }
    }

    It 'Dsc can process large resource output' -Pending {
        try {
            $env:TestClassResourceResultCount = 5000 # with sync resource invocations this was not possible

            $r = dsc -l trace resource export -r TestClassResource/TestClassResource 2> $TestDrive/tracing.txt
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/tracing.txt | Out-String)
            $res = $r | ConvertFrom-Json
            $res.resources[0].properties.result.count | Should -Be 5000
        }
        finally {
            $env:TestClassResourceResultCount = $null
        }
    }

    It 'Verify that there are no cache rebuilds for several sequential executions' {

        # remove cache file
        $cacheFilePath = if ($IsWindows) {
            # PS 6+ on Windows
            Join-Path $env:LocalAppData "dsc\PSAdapterCache.json"
        }
        else {
            # either WinPS or PS 6+ on Linux/Mac
            if ($PSVersionTable.PSVersion.Major -le 5) {
                Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"
            }
            else {
                Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
            }
        }
        Remove-Item -Force -Path $cacheFilePath -ErrorAction Ignore

        # first execution should build the cache
        dsc -l trace resource list -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Constructing Get-DscResource cache'

        # next executions following shortly after should Not rebuild the cache
        1..3 | ForEach-Object {
            dsc -l trace resource list -a Microsoft.DSC/PowerShell 2> $TestDrive/tracing.txt
            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Constructing Get-DscResource cache'
        }
    }

    It 'Can process a key-value pair object' {
        $r = '{"HashTableProp":{"Name":"DSCv3"},"Name":"TestClassResource1"}' | dsc resource get -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.HashTableProp.Name | Should -Be 'DSCv3'
    }

    It 'Specifying version works' {
        $out = dsc resource get -r TestClassResource/TestClassResource --version 0.0.1 | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.Ensure | Should -BeExactly 'Present'
    }

    It 'Specifying a non-existent version returns an error' {
        $null = dsc resource get -r TestClassResource/TestClassResource --version 0.0.2 2> $TestDrive/error.log
        $LASTEXITCODE | Should -Be 7
        (Get-Content -Raw -Path $TestDrive/error.log) | Should -BeLike '*Resource not found: TestClassResource/TestClassResource 0.0.2*' -Because (Get-Content -Raw -Path $TestDrive/error.log)
    }

    It 'Can process SecureString property' {
        $r = '{"Name":"TestClassResource1","SecureStringProp":"MySecretValue"}' | dsc resource get -r 'TestClassResource/TestClassResource' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.SecureStringProp | Should -Not -BeNullOrEmpty
    }

    Context 'Tracing works' {
        It 'Error messages come from Write-Error' {
            $null = dsc -l error resource set -r TestClassResource/TestClassResource -i '{"Name":"TestClassResource1"}' 2> $TestDrive/error.log
            $logContent = Get-Content -Path $TestDrive/error.log -Raw
            $LASTEXITCODE | Should -Be 2 -Because $logContent
            $logContent | Should -Match 'ERROR .*? This is an Error message' -Because $logContent
        }

        It 'Warning messages come from Write-Warning' {
            $null = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc -l warn resource test -r 'TestClassResource/TestClassResource' -f - 2> $TestDrive/warning.log
            $logContent = Get-Content -Path $TestDrive/warning.log -Raw
            $LASTEXITCODE | Should -Be 0 -Because $logContent
            $logContent | Should -Match 'WARN .*? This is a Warning message' -Because $logContent
        }

        It 'Info messages come from Write-Host' {
            $null = "{'Name':'TestClassResource1'}" | dsc -l info resource set -r 'TestClassResource/TestClassResource' -f - 2> $TestDrive/verbose.log
            $logContent = Get-Content -Path $TestDrive/verbose.log -Raw
            $LASTEXITCODE | Should -Be 2 -Because $logContent
            $logContent | Should -Match 'INFO .*? This is a Host message' -Because $logContent
        }

        It 'Debug messages come from Write-Verbose' {
            $null = "{'Name':'TestClassResource1'}" | dsc -l debug resource get -r 'TestClassResource/TestClassResource' -f - 2> $TestDrive/debug.log
            $logContent = Get-Content -Path $TestDrive/debug.log -Raw
            $LASTEXITCODE | Should -Be 0 -Because $logContent
            $logContent | Should -Match 'DEBUG .*? This is a Verbose message' -Because $logContent
        }

        It 'Trace messages come from Write-Debug' {
            $null = dsc -l trace resource export -r TestClassResource/TestClassResource 2> $TestDrive/trace.log
            $logContent = Get-Content -Path $TestDrive/trace.log -Raw
            $LASTEXITCODE | Should -Be 2 -Because $logContent
            $logContent | Should -Match 'TRACE .*? This is a Debug message' -Because $logContent
        }

        It 'Trace messages come from Write-Information' {
            $null = dsc -l trace resource set -r TestClassResource/TestClassResource -i '{"Name":"TestClassResource1"}' 2> $TestDrive/trace_info.log
            $logContent = Get-Content -Path $TestDrive/trace_info.log -Raw
            $LASTEXITCODE | Should -Be 2 -Because $logContent
            $logContent | Should -Match 'TRACE .*? This is an Information message' -Because $logContent
        }
    }
}
