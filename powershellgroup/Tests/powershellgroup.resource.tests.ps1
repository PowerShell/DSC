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

    It 'Discovery includes class-based and script-based resources ' {
        
        $r = dsc resource list
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | ? {$_.Type -eq 'PSTestModule/TestClassResource'}).Count | Should -Be 1
        ($resources | ? {$_.Type -eq 'PSTestModule/TestPSRepository'}).Count | Should -Be 1
    }

    It 'Get works on class-based resource' {
        
        $r = "{'Name':'TestClassResource1'}" | dsc.exe resource get -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actual_state.Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Get works on script-based resource' {
        
        $r = "{'Name':'TestPSRepository1'}" | dsc.exe resource get -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actual_state.PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
    }

    It 'Test works on class-based resource' {
        
        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc.exe resource test -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actual_state.InDesiredState | Should -Be $True
    }

    It 'Test works on script-based resource' {
        
        $r = "{'Name':'TestPSRepository1','PackageManagementProvider':'NuGet'}" | dsc.exe resource test -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actual_state.InDesiredState | Should -Be $True
    }

    It 'Set works on class-based resource' {
        
        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1'}" | dsc.exe resource set -r PSTestModule/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.after_state.RebootRequired | Should -Not -BeNull
    }

    It 'Set works on script-based resource' {
        
        $r = "{'Name':'TestPSRepository1'}" | dsc.exe resource set -r PSTestModule/TestPSRepository
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.after_state.RebootRequired | Should -Not -BeNull
    }
}
