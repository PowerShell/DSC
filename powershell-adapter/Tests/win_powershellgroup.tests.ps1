# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WindowsPowerShell adapter resource tests' {

    BeforeAll {
        if ($isWindows) {
            winrm quickconfig -quiet -force
        }    
        $OldPSModulePath  = $env:PSModulePath
        $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot

        $winpsConfigPath = Join-path $PSScriptRoot "winps_resource.dsc.yaml"
        if ($isWindows) {
            $cacheFilePath_v5 = Join-Path $env:LocalAppData "dsc" "WindowsPSAdapterCache.json"
        }
    }
    AfterAll {
        $env:PSModulePath = $OldPSModulePath
    }

    BeforeEach {
        if ($isWindows) {
            Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath_v5
        }
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

    It 'Set works on Binary "File" resource' -Skip:(!$IsWindows){

        $testFile = "$testdrive\test.txt"
        $r = '{"DestinationPath":"' + $testFile.replace('\','\\') + '", type: File, contents: HelloWorld, Ensure: present}' | dsc resource set -r 'PSDesiredStateConfiguration/File'
        $LASTEXITCODE | Should -Be 0
        Get-Content -Raw -Path $testFile | Should -Be "HelloWorld"
    }

    It 'Get works on traditional "Script" resource' -Skip:(!$IsWindows){

        $testFile = "$testdrive\test.txt"
        'test' | Set-Content -Path $testFile -Force
        $r = '{"GetScript": "@{result = $(Get-Content ' + $testFile.replace('\','\\') + ')}", "SetScript": "throw", "TestScript": "throw"}' | dsc resource get -r 'PSDesiredStateConfiguration/Script'
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.result | Should -Be 'test'
    }

    It 'Get works on config with File resource for WinPS' -Skip:(!$IsWindows){

      $testFile = "$testdrive\test.txt"
      'test' | Set-Content -Path $testFile -Force
      $r = (Get-Content -Raw $winpsConfigPath).Replace('c:\test.txt',"$testFile") | dsc config get
      $LASTEXITCODE | Should -Be 0
      $res = $r | ConvertFrom-Json
      $res.results[0].result.actualState.result[0].properties.DestinationPath | Should -Be "$testFile"
    }
}
