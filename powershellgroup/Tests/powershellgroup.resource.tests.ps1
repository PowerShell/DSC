# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShellGroup resource tests' {

    BeforeAll {
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += ";" + $PSScriptRoot
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    It 'Discovery includes class-based and script-based resources ' -Skip:(!$IsWindows){

        $r = dsc resource list
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | ? {$_.Type -eq 'PSTestModule/TestClassResource'}).Count | Should -Be 1
        ($resources | ? {$_.Type -eq 'PSTestModule/TestPSRepository'}).Count | Should -Be 1
    }

    It 'Get works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.resources[0].properties.Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Get works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource get -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.resources[0].properties.PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
    }

    It 'Get uses enum names on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.resources[0].properties.EnumProp | Should -BeExactly 1
    }

    It 'Get uses enum names on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource get -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.resources[0].properties.Ensure | Should -BeExactly 1
    }

    It 'Test works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource test -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        write-verbose $res -verbose
        $res.actualState.resources[0].properties.InDesiredState | Should -Be $True
    }

    It 'Test works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1','PackageManagementProvider':'NuGet'}" | dsc resource test -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.resources[0].properties.InDesiredState | Should -Be $True
    }

    It 'Set works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource set -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.resources[0].properties.RebootRequired | Should -Not -BeNull
    }

    It 'Set works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource set -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.resources[0].properties.RebootRequired | Should -Not -BeNull
    }
}
