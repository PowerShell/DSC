# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WindowsPowerShell adapter resource tests - requires elevated permissions' {

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
        $r = '{"DestinationPath":"' + $testFile.replace('\','\\') + '"}' | dsc resource get -r 'PSDesiredStateConfiguration/File' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.DestinationPath | Should -Be "$testFile"
    }

    It 'Set works on Binary "File" resource' -Skip:(!$IsWindows){

        $testFile = "$testdrive\test.txt"
        $null = '{"DestinationPath":"' + $testFile.replace('\','\\') + '", type: File, contents: HelloWorld, Ensure: present}' | dsc resource set -r 'PSDesiredStateConfiguration/File' -f -
        $LASTEXITCODE | Should -Be 0
        Get-Content -Raw -Path $testFile | Should -Be "HelloWorld"
    }

    It 'Get works on traditional "Script" resource' -Skip:(!$IsWindows){

        $testFile = "$testdrive\test.txt"
        'test' | Set-Content -Path $testFile -Force
        $r = '{"GetScript": "@{result = $(Get-Content ' + $testFile.replace('\','\\') + ')}", "SetScript": "throw", "TestScript": "throw"}' | dsc resource get -r 'PSDesiredStateConfiguration/Script' -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.actualState.result.properties.result | Should -Be 'test'
    }

    It 'Get works on config with File resource for WinPS' -Skip:(!$IsWindows){

        $testFile = "$testdrive\test.txt"
        'test' | Set-Content -Path $testFile -Force
        $r = (Get-Content -Raw $winpsConfigPath).Replace('c:\test.txt',"$testFile") | dsc config get -f -
        $LASTEXITCODE | Should -Be 0
        $res = $r | ConvertFrom-Json
        $res.results[0].result.actualState.result[0].properties.DestinationPath | Should -Be "$testFile"
    }

    It 'Verify that there are no cache rebuilds for several sequential executions' -Skip:(!$IsWindows) {

        # remove cache file
        $cacheFilePath = Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"
        Remove-Item -Force -Path $cacheFilePath -ErrorAction Ignore

        # first execution should build the cache
        dsc -l trace resource list -a Microsoft.Windows/WindowsPowerShell 2> $TestDrive/tracing.txt
        "$TestDrive/tracing.txt" | Should -FileContentMatchExactly 'Constructing Get-DscResource cache'

        # next executions following shortly after should Not rebuild the cache
        1..3 | %{
            dsc -l trace resource list -a Microsoft.Windows/WindowsPowerShell 2> $TestDrive/tracing.txt
            "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Constructing Get-DscResource cache'
        }
    }

    It '_inDesiredState is returned correction: <Context>' -Skip:(!$IsWindows) -TestCases @(
        @{ Context = 'Both running'; FirstState = 'Running'; SecondState = 'Running' }
        @{ Context = 'Both stopped'; FirstState = 'Stopped'; SecondState = 'Stopped' }
        @{ Context = 'First Stopped'; FirstState = 'Stopped'; SecondState = 'Running' }
        @{ Context = 'First Running'; FirstState = 'Running'; SecondState = 'Stopped' }
      ) {
          param($Context, $FirstState, $SecondState)
          $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Use Windows PowerShell resources
    type: Microsoft.Windows/WindowsPowerShell
    properties:
      resources:
      - name: Check Spooler service 1
        type: PsDesiredStateConfiguration/Service
        properties:
          Name: Spooler
          State: $FirstState
      - name: Check Spooler service 2
        type: PsDesiredStateConfiguration/Service
        properties:
          Name: Spooler
          State: $SecondState    
"@

        $inDesiredState = if ($FirstState -eq $SecondState) {
          $FirstState -eq (Get-Service Spooler).Status
        } else {
          $false
        }

        $out = dsc config test -i $yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.inDesiredState | Should -Be $inDesiredState
    }
}
