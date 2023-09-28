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
        $res.actualState.Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Get works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource get -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
    }

    It 'Get uses enum names on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1'}" | dsc resource get -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.EnumProp | Should -BeExactly 'Expected'
    }

    It 'Get uses enum names on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource get -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.Ensure | Should -BeExactly 'Present'
    }

    It 'Test works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource test -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.InDesiredState | Should -Be $True
    }

    It 'Test works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1','PackageManagementProvider':'NuGet'}" | dsc resource test -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.InDesiredState | Should -Be $True
    }

    It 'Set works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc resource set -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.RebootRequired | Should -Not -BeNull
    }

    It 'Set works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1'}" | dsc resource set -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.RebootRequired | Should -Not -BeNull
    }
}
