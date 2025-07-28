# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'WindowsPowerShell adapter resource tests - requires elevated permissions' {

  BeforeAll {
    if ($isWindows) {
      winrm quickconfig -quiet -force
      $OldPSModulePath = $env:PSModulePath
      $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot

      $winpsConfigPath = Join-path $PSScriptRoot "winps_resource.dsc.yaml"
      if ($isWindows) {
        $cacheFilePath_v5 = Join-Path $env:LocalAppData "dsc" "WindowsPSAdapterCache.json"
      }
    }
  }
  AfterAll {
    if ($isWindows) {
      $env:PSModulePath = $OldPSModulePath

      # Remove after all the tests are done
      Remove-Module $script:winPSModule -Force -ErrorAction Ignore
    }
  }

  BeforeEach {
    if ($isWindows) {
      Remove-Item -Force -ea SilentlyContinue -Path $cacheFilePath_v5
    }
  }

  It 'Windows PowerShell adapter supports File resource' -Skip:(!$IsWindows) {

    $r = dsc resource list --adapter Microsoft.Windows/WindowsPowerShell
    $LASTEXITCODE | Should -Be 0
    $resources = $r | ConvertFrom-Json
    ($resources | Where-Object { $_.Type -eq 'PSDesiredStateConfiguration/File' }).Count | Should -Be 1
  }

  It 'Get works on Binary "File" resource' -Skip:(!$IsWindows) {

    $testFile = "$testdrive\test.txt"
    'test' | Set-Content -Path $testFile -Force
    $r = '{"DestinationPath":"' + $testFile.replace('\', '\\') + '"}' | dsc resource get -r 'PSDesiredStateConfiguration/File' -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.actualState.DestinationPath | Should -Be "$testFile"
  }

  It 'Set works on Binary "File" resource' -Skip:(!$IsWindows) {

    $testFile = "$testdrive\test.txt"
    $null = '{"DestinationPath":"' + $testFile.replace('\', '\\') + '", type: File, contents: HelloWorld, Ensure: present}' | dsc resource set -r 'PSDesiredStateConfiguration/File' -f -
    $LASTEXITCODE | Should -Be 0
    Get-Content -Raw -Path $testFile | Should -Be "HelloWorld"
  }

  It 'Get works on traditional "Script" resource' -Skip:(!$IsWindows) {

    $testFile = "$testdrive\test.txt"
    'test' | Set-Content -Path $testFile -Force
    $r = '{"GetScript": "@{result = $(Get-Content ' + $testFile.replace('\', '\\') + ')}", "SetScript": "throw", "TestScript": "throw"}' | dsc resource get -r 'PSDesiredStateConfiguration/Script' -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.actualState.result | Should -Be 'test'
  }

  It 'Get works on config with File resource for WinPS' -Skip:(!$IsWindows) {

    $testFile = "$testdrive\test.txt"
    'test' | Set-Content -Path $testFile -Force
    $r = (Get-Content -Raw $winpsConfigPath).Replace('c:\test.txt', "$testFile") | dsc config get -f -
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
    1..3 | ForEach-Object {
      dsc -l trace resource list -a Microsoft.Windows/WindowsPowerShell 2> $TestDrive/tracing.txt
      "$TestDrive/tracing.txt" | Should -Not -FileContentMatchExactly 'Constructing Get-DscResource cache'
    }
  }

  It 'Verify if assertion is used that no module is cleared in the cache' -Skip:(!$IsWindows) {
    # create a test file in the test drive
    $testFile = "$testdrive\test.txt"
    New-Item -Path $testFile -ItemType File -Force | Out-Null

    # remove cache file
    $cacheFilePath = Join-Path $env:LocalAppData "dsc\WindowsPSAdapterCache.json"
    Remove-Item -Force -Path $cacheFilePath -ErrorAction Ignore

    # build the cache
    dsc resource list --adapter Microsoft.Windows/WindowsPowerShell | Out-Null

    # Create a test module in the test drive
    $testModuleDir = "$testdrive\TestModule\1.0.0"
    New-Item -Path $testModuleDir -ItemType Directory -Force | Out-Null

    $manifestContent = @"
        @{
            RootModule = 'TestModule.psm1'
            ModuleVersion = '1.0.0'
            GUID = $([guid]::NewGuid().Guid)
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
    $env:PSModulePath += [System.IO.Path]::PathSeparator + $testdrive

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

    $cache = Get-Content -Path $cacheFilePath -Raw | ConvertFrom-Json
    $cache.ResourceCache.Type | Should -Contain 'PSTestModule/TestPSRepository'
    $cache.ResourceCache.Type | Should -Contain 'PSDesiredStateConfiguration/File'
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
    }
    else {
      $false
    }

    $out = dsc config test -i $yaml | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.inDesiredState | Should -Be $inDesiredState
  }

  # TODO: fix test - currently throwing an exception, -Skip:(!$IsWindows)
  It 'Config works with credential object' -Skip {
    BeforeDiscovery {
      $script:winPSModule = Resolve-Path -Path (Join-Path $PSScriptRoot '..' 'psDscAdapter' 'win_psDscAdapter.psm1') | Select-Object -ExpandProperty Path
      Import-Module $winPSModule -Force -ErrorAction Stop

      # Mock the command to work on GitHub runners because Microsoft.PowerShell.Security is not available
      Mock -CommandName ConvertTo-SecureString -MockWith { [System.Security.SecureString]::new() }
    }

    $jsonInput = @{
      resources = @{
        name       = 'Service info'
        type       = 'PSDesiredStateConfiguration/Service'
        properties = @{
          Name       = 'Spooler'
          Credential = @{
            UserName = 'User'
            Password = 'Password'
          }
        }
      }
    } | ConvertTo-Json -Depth 10

    # Instead of calling dsc.exe we call the cmdlet directly to be able to test the output and mocks
    $resourceObject = Get-DscResourceObject -jsonInput $jsonInput
    $cacheEntry = Invoke-DscCacheRefresh -Module PSDesiredStateConfiguration

    $out = Invoke-DscOperation -Operation Test -DesiredState $resourceObject -dscResourceCache $cacheEntry
    $LASTEXITCODE | Should -Be 0
    $out.properties.InDesiredState.InDesiredState | Should -Be $false
    Should -Invoke -CommandName ConvertTo-SecureString -Exactly -Times 1 -Scope It
  }

  It 'Config does not work when credential properties are missing required fields' -Skip:(!$IsWindows) {
    $yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Service info
          type: PsDesiredStateConfiguration/Service
          properties:
            Name: Spooler
            Credential:
              UserName: 'User'
              OtherProperty: 'Password'
"@
    # Compared to PowerShell we use test here as it filters out the properties
    $out = dsc config test -i $yaml 2>&1 | Out-String
    $LASTEXITCODE | Should -Be 2
    $out | Should -Not -BeNullOrEmpty
    $out | Should -BeLike "*ERROR*Credential object 'Credential' requires both 'username' and 'password' properties*"
  }

  It 'List works with class-based PS DSC resources' -Skip:(!$IsWindows) {
    BeforeDiscovery {
      $windowsPowerShellPath = Join-Path $testDrive 'WindowsPowerShell' 'Modules'
      $env:PSModulePath += [System.IO.Path]::PathSeparator + $windowsPowerShellPath

      $moduleFile = @"
@{
    RootModule           = 'PSClassResource.psm1'
    ModuleVersion        = '0.1.0'
    GUID                 = '1b2e177b-1819-4f51-8bc9-795dd8fae984'
    Author               = 'Microsoft Corporation'
    CompanyName          = 'Microsoft Corporation'
    Copyright            = '(c) Microsoft Corporation. All rights reserved.'
    Description          = 'DSC Resource for Windows PowerShell Class'
    PowerShellVersion    = '5.1'
    DscResourcesToExport = @(
        'PSClassResource'
    )
    PrivateData          = @{
        PSData = @{
            Tags       = @(
                'PSDscResource_PSClassResource'
            )
            DscCapabilities = @(
            'get'
            'test'
            'set'
            'export'
            )
        }
    }
}
"@
      $moduleFilePath = Join-Path $windowsPowerShellPath 'PSClassResource' '0.1.0' 'PSClassResource.psd1'
      if (-not (Test-Path -Path $moduleFilePath)) {
        New-Item -Path $moduleFilePath -ItemType File -Value $moduleFile -Force | Out-Null
      }


      $module = @'
enum Ensure {
    Present
    Absent
}

[DSCResource()]
class PSClassResource {
    [DscProperty(Key)]
    [string] $Name

    [string] $NonDscProperty

    hidden
    [string] $HiddenNonDscProperty

    [DscProperty()]
    [Ensure] $Ensure = [Ensure]::Present

    PSClassResource() {
    }

    [PSClassResource] Get() {
        return $this
    }

    [bool] Test() {
        return $true
    }

    [void] Set() {

    }

    static [PSClassResource[]] Export()
    {
        $resultList = [System.Collections.Generic.List[PSClassResource]]::new()
        $resultCount = 5
        if ($env:PSClassResourceResultCount) {
            $resultCount = $env:PSClassResourceResultCount
        }
        1..$resultCount | %{
            $obj = New-Object PSClassResource
            $obj.Name = "Object$_"
            $obj.Ensure = [Ensure]::Present
            $resultList.Add($obj)
        }

        return $resultList.ToArray()
    }
}
'@

      $modulePath = Join-Path $windowsPowerShellPath 'PSClassResource' '0.1.0' 'PSClassResource.psm1'
      if (-not (Test-Path -Path $modulePath)) {
        New-Item -Path $modulePath -ItemType File -Value $module -Force | Out-Null
      }
    }

    $out = dsc -l trace resource list --adapter Microsoft.Windows/WindowsPowerShell | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.type | Should -Contain 'PSClassResource/PSClassResource'
    $out | Where-Object -Property type -EQ PSClassResource/PSClassResource | Select-Object -ExpandProperty implementedAs | Should -Be 1 # Class-based
    ($out | Where-Object -Property type -EQ 'PSClassResource/PSClassResource').capabilities | Should -BeIn @('get', 'test', 'set', 'export')
  }

  It 'Get works with class-based PS DSC resources' -Skip:(!$IsWindows) {

    $out = dsc resource get -r PSClassResource/PSClassResource --input (@{Name = 'TestName' } | ConvertTo-Json) | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.actualState.Name | Should -Be 'TestName'
    $propCount = $out.actualState | Get-Member -MemberType NoteProperty
    $propCount.Count | Should -Be 1 # Only the DscProperty should be returned
  }

  It 'Set works with class-based PS DSC resources' -Skip:(!$IsWindows) {

    $out = dsc resource set -r PSClassResource/PSClassResource --input (@{Name = 'TestName' } | ConvertTo-Json) | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.afterstate.InDesiredState | Should -Be $true
  }

  It 'Export works with class-based PS DSC resources' -Skip:(!$IsWindows) {

    $out = dsc resource export -r PSClassResource/PSClassResource | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out | Should -Not -BeNullOrEmpty
    $out.resources.count | Should -Be 5
    $out.resources[0].properties.Ensure | Should -Be 'Present' # Check for enum property
  }
}

