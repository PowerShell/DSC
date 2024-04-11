# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter resource tests' {

    BeforeAll {
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    It 'Discovery includes class-based and script-based resources ' -Skip:(!$IsWindows){

        $r = dsc resource list * -a *PowerShell*
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | ? {$_.Type -eq 'TestClassResource/TestClassResource'}).Count | Should -Be 1
        ($resources | ? {$_.Type -eq 'PSTestModule/TestPSRepository'}).Count | Should -Be 1
    }

    It 'Windows PowerShell adapter supports File resource' -Skip:(!$IsWindows){

        $r = dsc resource list --adapter Microsoft.DSC/WindowsPowerShell
        $LASTEXITCODE | Should -Be 0
        $resources = $r | ConvertFrom-Json
        ($resources | ? {$_.Type -eq 'PSDesiredStateConfiguration/File'}).Count | Should -Be 1
    }

    It 'Get works on Binary "File" resource' -Skip:(!$IsWindows){

        $testFile = 'c:\test.txt'
        'test' | Set-Content -Path $testFile -Force
        $r = '{"DestinationPath":"' + $testFile.replace('\','\\') + '"}' | dsc resource get -r 'PSDesiredStateConfiguration/File'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.DestinationPath | Should -Be "$testFile"
    }

    It 'Get works on traditional "Script" resource' -Skip:(!$IsWindows){

        $testFile = 'c:\test.txt'
        'test' | Set-Content -Path $testFile -Force
        $r = '{"GetScript": "@{result = $(Get-Content ' + $testFile.replace('\','\\') + ')}", "SetScript": "throw", "TestScript": "throw"}' | dsc resource get -r 'PSDesiredStateConfiguration/Script'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.result | Should -Be 'test'
    }

    It 'Get works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1', 'Type':'TestClassResource/TestClassResource'}" | dsc resource get -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.Prop1 | Should -BeExactly 'ValueForProp1'
    }

    It 'Get works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1','Type':'PSTestModule/TestPSRepository'}" | dsc resource get -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.PublishLocation | Should -BeExactly 'https://www.powershellgallery.com/api/v2/package/'
    }

    It 'Get uses enum names on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Type':'TestClassResource/TestClassResource'}" | dsc resource get -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.EnumProp | Should -BeExactly 'Expected'
    }

    It 'Get uses enum names on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1','Type':'PSTestModule/TestPSRepository'}" | dsc resource get -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.Ensure | Should -BeExactly 'Present'
    }

    <#
    It 'Test works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1','Type':'TestClassResource/TestClassResource'}" | dsc resource test -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.InDesiredState | Should -Be $True
    }

    It 'Test works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1','PackageManagementProvider':'NuGet','Type':'PSTestModule/TestPSRepository'}" | dsc resource test -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.InDesiredState | Should -Be $True
    }

    It 'Set works on class-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestClassResource1','Prop1':'ValueForProp1','Type':'TestClassResource/TestClassResource'}" | dsc resource set -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.RebootRequired | Should -Not -BeNull
    }

    It 'Set works on script-based resource' -Skip:(!$IsWindows){

        $r = "{'Name':'TestPSRepository1','Type':'PSTestModule/TestPSRepository'}" | dsc resource set -r 'Microsoft.Dsc/PowerShell'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.afterState.RebootRequired | Should -Not -BeNull
    }

    It 'Export works on PS class-based resource' -Skip:(!$IsWindows){

        $r = dsc resource export -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.resources.count | Should -Be 5
        $res.resources[0].type | Should -Be "TestClassResource/TestClassResource"
        $res.resources[0].properties.Name | Should -Be "Object1"
        $res.resources[0].properties.Prop1 | Should -Be "Property of object1"
    }

    It 'Get --all works on PS class-based resource' -Skip:(!$IsWindows){

        $r = dsc resource get --all -r TestClassResource/TestClassResource
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.count | Should -Be 5
        $res | % {$_.actualState | Should -Not -BeNullOrEmpty}
    }
    #>
}
