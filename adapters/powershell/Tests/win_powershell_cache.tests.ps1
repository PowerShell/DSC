# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
    }
}

Describe 'WindowsPowerShell adapter resource tests - requires elevated permissions' -Skip:(!$IsWindows -or !$isElevated) {

  BeforeAll {
    $OldPSModulePath = $env:PSModulePath
    $dscHome = Split-Path (Get-Command dsc -ErrorAction Stop).Source -Parent
    $psexeHome = Split-Path (Get-Command powershell -ErrorAction Stop).Source -Parent
    $ps7exeHome = Split-Path (Get-Command pwsh -ErrorAction Stop).Source -Parent
    $env:DSC_RESOURCE_PATH = $dscHome + [System.IO.Path]::PathSeparator + $psexeHome + [System.IO.Path]::PathSeparator + $ps7exeHome
    $null = winrm quickconfig -quiet -force 2>&1
    $env:PSModulePath = $PSScriptRoot + [System.IO.Path]::PathSeparator + $env:PSModulePath

    $winpsConfigPath = Join-path $PSScriptRoot "winps_resource.dsc.yaml"
    $cacheFilePath_v5 = Join-Path $env:LocalAppData "dsc" "WindowsPSAdapterCache.json"

    $script:winPSModule = Resolve-Path -Path (Join-Path $PSScriptRoot '..' 'psDscAdapter' 'win_psDscAdapter.psm1') | Select-Object -ExpandProperty Path
    Import-Module $winPSModule -Force -ErrorAction Stop
  }

  AfterAll {
    $env:PSModulePath = $OldPSModulePath
    $env:DSC_RESOURCE_PATH = $null

    # Remove after all the tests are done
    Remove-Module $script:winPSModule -Force -ErrorAction Ignore
  }

  BeforeEach {
    Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath_v5
  }

  It 'Windows PowerShell adapter supports File resource' {

    $r = dsc resource list --adapter Microsoft.Windows/WindowsPowerShell
    $LASTEXITCODE | Should -Be 0
    $resources = $r | ConvertFrom-Json
    ($resources | Where-Object { $_.Type -eq 'PSDesiredStateConfiguration/File' }).Count | Should -Be 1
  }

  It 'Get works on Binary "File" resource' {

    $testFile = "$testdrive\test.txt"
    'test' | Set-Content -Path $testFile -Force
    $r = '{"DestinationPath":"' + $testFile.replace('\', '\\') + '"}' | dsc resource get -r 'PSDesiredStateConfiguration/File' -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.actualState.DestinationPath | Should -Be "$testFile"
  }

  It 'Set works on Binary "File" resource' {

    $testFile = "$testdrive\test.txt"
    $null = '{"DestinationPath":"' + $testFile.replace('\', '\\') + '", type: File, contents: HelloWorld, Ensure: present}' | dsc resource set -r 'PSDesiredStateConfiguration/File' -f -
    $LASTEXITCODE | Should -Be 0
    Get-Content -Raw -Path $testFile | Should -Be "HelloWorld"
  }

  It 'Get works on traditional "Script" resource' {

    $testFile = "$testdrive\test.txt"
    'test' | Set-Content -Path $testFile -Force
    $r = '{"GetScript": "@{result = $(Get-Content ' + $testFile.replace('\', '\\') + ')}", "SetScript": "throw", "TestScript": "throw"}' | dsc resource get -r 'PSDesiredStateConfiguration/Script' -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.actualState.result | Should -Be 'test'
  }

  It 'Get works on config with File resource for WinPS' {

    $testFile = "$testdrive\test.txt"
    'test' | Set-Content -Path $testFile -Force
    $r = (Get-Content -Raw $winpsConfigPath).Replace('c:\test.txt', "$testFile") | dsc config get -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.results[0].result.actualState.result[0].properties.DestinationPath | Should -Be "$testFile"
  }

  It 'Verify that there are no cache rebuilds for several sequential executions' {
    # first execution should build the cache
    $null = dsc -l trace resource list -a Microsoft.Windows/WindowsPowerShell 2> $TestDrive/tracing.txt
    $tracingContent = Get-Content -Path $TestDrive/tracing.txt | Out-String
    $tracingContent | Should -BeLike '*Constructing Get-DscResource cache*' -Because $tracingContent

    # next executions following shortly after should Not rebuild the cache
    1..3 | ForEach-Object {
      $null = dsc -l trace resource list -a Microsoft.Windows/WindowsPowerShell 2> $TestDrive/tracing.txt
      $tracingContent = Get-Content -Path $TestDrive/tracing.txt | Out-String
      $tracingContent | Should -Not -BeLike '*Constructing Get-DscResource cache*' -Because $tracingContent
    }
  }

  It 'Verify if assertion is used that no module is cleared in the cache' {
    # create a test file in the test drive
    $testFile = "$testdrive\test.txt"
    New-Item -Path $testFile -ItemType File -Force | Out-Null

    # build the cache
    dsc resource list --adapter Microsoft.Windows/WindowsPowerShell | Out-Null

    # Create a test module in the test drive
    $testModuleDir = "$testdrive\TestModule\1.0.0"
    New-Item -Path $testModuleDir -ItemType Directory -Force | Out-Null

    $manifestContent = @"
        @{
            RootModule = 'TestModule.psm1'
            ModuleVersion = '1.0.0'
            GUID = '$([guid]::NewGuid().Guid)'
            Author = 'Microsoft Corporation'
            CompanyName = 'Microsoft Corporation'
            Copyright = '(c) Microsoft Corporation. All rights reserved.'
            Description = 'Test module for DSC tests'
            PowerShellVersion = '5.1'
            DscResourcesToExport = @()
            FunctionsToExport = @()
            CmdletsToExport = @()
            VariablesToExport = @()
            AliasesToExport = @()
        }
"@
    Set-Content -Path "$testModuleDir\TestModule.psd1" -Value $manifestContent

    $scriptContent = @"
Write-Host 'The DSC world!'
"@
    Set-Content -Path "$testModuleDir\TestModule.psm1" -Value $scriptContent

    # Add the test module directory to PSModulePath
    $env:PSModulePath = $testdrive + [System.IO.Path]::PathSeparator + $env:PSModulePath

    $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: File
    type: Microsoft.Windows/WindowsPowerShell
    properties:
      resources:
        - name: File
          type: PSDesiredStateConfiguration/File
          properties:
            DestinationPath: $testfile
  - name: File present
    type: Microsoft.DSC/Assertion
    properties:
      `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      resources:
        - name: Use powershell adapter
          type: Microsoft.Windows/WindowsPowerShell
          properties:
            resources:
              - name: File present
                type: PSDesiredStateConfiguration/File
                properties:
                  DestinationPath: $testFile
    dependsOn:
      - "[resourceId('Microsoft.Windows/WindowsPowerShell', 'File')]"
  - name: TestPSRepository
    type: PSTestModule/TestPSRepository
    properties:
      Name: NuGet
    dependsOn:
      - "[resourceId('Microsoft.Windows/WindowsPowerShell', 'File')]"
      - "[resourceId('Microsoft.DSC/Assertion', 'File present')]"
"@
    # output to file for Windows PowerShell 5.1
    $filePath = "$testdrive\test.assertion.dsc.resource.yaml"
    $yaml | Set-Content -Path $filePath -Force
    dsc config test -f $filePath 2> "$TestDrive/error.txt"
    $LASTEXITCODE | Should -Be 2

    $cache = Get-Content -Path $cacheFilePath_v5 -Raw | ConvertFrom-Json
    $cache.ResourceCache.Type | Should -Contain 'PSTestModule/TestPSRepository'
    $cache.ResourceCache.Type | Should -Contain 'PSDesiredStateConfiguration/File'
  }
}
