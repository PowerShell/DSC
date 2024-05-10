# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter resource tests' {

    BeforeAll {
        if ($isWindows) {
            winrm quickconfig -quiet -force
        }    
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot

        if ($IsLinux -or $IsMacOS) {
            $cacheFilePath = Join-Path $env:HOME "dsc" "PSAdapterCache.json"
        }
        else
        {
            $cacheFilePath = Join-Path $env:LocalAppData "dsc" "PSAdapterCache.json"
            $cacheFilePath_v5 = Join-Path $env:LocalAppData "dsc" "WindowsPSAdapterCache.json"
        }
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    BeforeEach {
        Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath
        Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath_v5
    }

    It 'Discovery includes class-based and script-based resources ' -Skip:(!$IsWindows){

        $r = dsc resource list * -a Microsoft.DSC/PowerShell
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | ? {$_.Type -eq 'TestClassResource/TestClassResource'}).Count | Should -Be 1
        ($resources | ? {$_.Type -eq 'PSTestModule/TestPSRepository'}).Count | Should -Be 1
    }

    It 'Windows PowerShell adapter supports File resource' -Skip:(!$IsWindows){

        $r = dsc resource list --adapter Microsoft.Windows/WindowsPowerShell
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | ? {$_.Type -eq 'PSDesiredStateConfiguration/File'}).Count | Should -Be 1
    }

    It 'Get works on Binary "File" resource' -Skip:(!$IsWindows){

        $testFile = "$testdrive\test.txt"
        'test' | Set-Content -Path $testFile -Force
        $r = '{"DestinationPath":"' + $testFile.replace('\','\\') + '"}' | dsc resource get -r 'PSDesiredStateConfiguration/File'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.DestinationPath | Should -Be "$testFile"
    }

    It 'Get works on traditional "Script" resource' -Skip:(!$IsWindows){

        $testFile = "$testdrive\test.txt"
        'test' | Set-Content -Path $testFile -Force
        $r = '{"GetScript": "@{result = $(Get-Content ' + $testFile.replace('\','\\') + ')}", "SetScript": "throw", "TestScript": "throw"}' | dsc resource get -r 'PSDesiredStateConfiguration/Script'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.result | Should -Be 'test'
    }

    It 'Get works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Get works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource get -r 'PSTestModule/TestPSRepository'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
    }

    It 'Get uses enum names on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r 'TestClassResource/TestClassResource'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.EnumProp | Should -BeExactly 'Expected'
    }

    It 'Get uses enum names on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource get -r 'PSTestModule/TestPSRepository'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.Ensure | Should -BeExactly 'Present'
    }

    It 'Test works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource test -r 'TestClassResource/TestClassResource'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.InDesiredState | Should -Be $True
    }

    It 'Test works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1','PackageManagementProvider':'NuGet'}" | dsc resource test -r 'PSTestModule/TestPSRepository'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.InDesiredState | Should -Be $True
    }

    It 'Set works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource set -r 'TestClassResource/TestClassResource'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.result | Should -Not -BeNull
    }

    It 'Set works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource set -r 'PSTestModule/TestPSRepository'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.result.properties.RebootRequired | Should -Not -BeNull
    }

    It 'Export works on PS class-based resource' -Skip:(!$IsWindows){

        $r = dsc resource export -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.resources[0].properties.result.count | Should -Be 5
        $res.resources[0].properties.result[0].Name | Should -Be "Object1"
        $res.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"
    }

    It 'Get --all works on PS class-based resource' -Skip:(!$IsWindows){

        $r = dsc resource get --all -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.count | Should -Be 5
        $res.actualState.result| % {$_.Name | Should -Not -BeNullOrEmpty}
    }
}
