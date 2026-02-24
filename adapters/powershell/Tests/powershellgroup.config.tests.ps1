# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter resource tests' {

  BeforeAll {
    $OldPSModulePath = $env:PSModulePath
    $env:PSModulePath += [System.IO.Path]::PathSeparator + $PSScriptRoot
    $pwshConfigPath = Join-path $PSScriptRoot "class_ps_resources.dsc.yaml"

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

  It 'Get works on config with class-based resources' {

    $r = Get-Content -Raw $pwshConfigPath | dsc config get -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.results[0].result.actualState.result[0].properties.Prop1 | Should -BeExactly 'ValueForProp1'
    $res.results[0].result.actualState.result[0].properties.EnumProp | Should -BeExactly 'Expected'
  }

  It 'Get does not work on config when module does not exist' {

    $yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResourceNotExist/TestClassResourceNotExist
'@
    $yaml | dsc -l trace config get -f - 2> "$TestDrive/tracing.txt"
    $LASTEXITCODE | Should -Be 2
    "$TestDrive/tracing.txt" | Should -FileContentMatch "DSC resource 'TestClassResourceNotExist/TestClassResourceNotExist' module not found."
  }

  It 'Test works on config with class-based resources' {

    $r = Get-Content -Raw $pwshConfigPath | dsc config test -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.results[0].result.actualState.result[0] | Should -Not -BeNull
  }

  It 'Set works on config with class-based resources' {

    $r = Get-Content -Raw $pwshConfigPath | dsc config set -f -
    $LASTEXITCODE | Should -Be 0
    $res = $r | ConvertFrom-Json
    $res.results.result.afterState.result[0].type | Should -Be "TestClassResource/TestClassResource"
  }

  It 'Export works on config with class-based resources' {

    $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
'@
    $out = $yaml | dsc config export -f -
    $LASTEXITCODE | Should -Be 0
    $res = $out | ConvertFrom-Json
    $res.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
    $res.'resources' | Should -Not -BeNullOrEmpty
    $res.resources[0].properties.result.count | Should -Be 5
    $res.resources[0].properties.result[0].Name | Should -Be "Object1"
    $res.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"
  }

  It 'Export fails when class-based resource does not implement' {
    $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/NoExport
'@
    $out = $yaml | dsc config export -f - 2>&1 | Out-String
    $LASTEXITCODE | Should -Be 2
    $out | Should -Not -BeNullOrEmpty
    $out | Should -BeLike "*ERROR*Export method not implemented by resource 'TestClassResource/NoExport'*"
  }

  It 'Export works with filtered export property' {
    $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/FilteredExport
                  properties:
                    Name: 'FilteredExport'
'@
    $out = $yaml | dsc -l trace config export -f - 2> "$TestDrive/export_trace.txt"
    $LASTEXITCODE | Should -Be 0
    $res = $out | ConvertFrom-Json
    $res.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
    $res.'resources' | Should -Not -BeNullOrEmpty
    $res.resources[0].properties.result.count | Should -Be 1
    $res.resources[0].properties.result[0].Name | Should -Be "FilteredExport"
    $res.resources[0].properties.result[0].Prop1 | Should -Be "Filtered Property for FilteredExport"
    "$TestDrive/export_trace.txt" | Should -FileContentMatch "Properties provided for filtered export"
  }

  It 'Export fails when filtered export is requested but not implemented' {
    $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/NoExport
                  properties:
                    Name: 'SomeFilter'
'@
    $out = $yaml | dsc config export -f - 2>&1 | Out-String
    $LASTEXITCODE | Should -Be 2
    $out | Should -Not -BeNullOrEmpty
    $out | Should -BeLike "*ERROR*Export method with parameters not implemented by resource 'TestClassResource/NoExport'*"
  }

  It 'Custom psmodulepath in config works' {

    $OldPSModulePath = $env:PSModulePath
    Copy-Item -Recurse -Force -Path "$PSScriptRoot/TestClassResource" -Destination $TestDrive
    Rename-Item -Path "$PSScriptRoot/TestClassResource" -NewName "_TestClassResource"

    try {
      $psmp = "`$env:PSModulePath" + [System.IO.Path]::PathSeparator + $TestDrive
      $yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                psmodulepath: $psmp
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
"@
      $out = $yaml | dsc config export -f -
      $LASTEXITCODE | Should -Be 0
      $res = $out | ConvertFrom-Json
      $res.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
      $res.'resources' | Should -Not -BeNullOrEmpty
      $res.resources[0].properties.result.count | Should -Be 5
      $res.resources[0].properties.result[0].Name | Should -Be "Object1"
      $res.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"
    }
    finally {
      Rename-Item -Path "$PSScriptRoot/_TestClassResource" -NewName "TestClassResource"
      $env:PSModulePath = $OldPSModulePath
    }
  }

  It 'DSCConfigRoot macro is working when config is from a file' {

    $yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
                  properties:
                    Name: "[envvar('DSC_CONFIG_ROOT')]"
"@

    $config_path = "$TestDrive/test_config.dsc.yaml"
    $yaml | Set-Content -Path $config_path

    $out = dsc config get --file $config_path
    $LASTEXITCODE | Should -Be 0
    $res = $out | ConvertFrom-Json
    $res.results.result.actualState.result.properties.Name | Should -Be $TestDrive
    $res.results.result.actualState.result.properties.Prop1 | Should -Be $TestDrive
  }

  It 'DSC_CONFIG_ROOT env var is cwd when config is piped from stdin' {

    $yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
                  properties:
                    Name: "[envvar('DSC_CONFIG_ROOT')]"
"@
    $out = $yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.result[0].properties.Name | Should -BeExactly (Get-Location).Path
  }

  It 'DSC Configuration Document with key-value pair works' {
    $yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Working with class-based resources
              type: Microsoft.DSC/PowerShell
              properties:
                resources:
                - name: Class-resource Info
                  type: TestClassResource/TestClassResource
                  properties:
                    Name: 'TestClassResource1'
                    HashTableProp:
                      Name: 'DSCv3'
"@

    $out = $yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results.result.actualState.result.properties.HashTableProp.Name | Should -BeExactly 'DSCv3'
  }

  It 'Config calling PS Resource directly works for <operation> with metadata <metadata> and adapter <adapter>' -TestCases @(
    @{ Operation = 'get'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.DSC/PowerShell' }
    @{ Operation = 'set'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.DSC/PowerShell' }
    @{ Operation = 'test'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.DSC/PowerShell' }
    @{ Operation = 'get'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Adapter/PowerShell' }
    @{ Operation = 'set'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Adapter/PowerShell' }
    @{ Operation = 'test'; metadata = 'Microsoft.DSC'; adapter = 'Microsoft.Adapter/PowerShell' }
    @{ Operation = 'get'; metadata = 'Ignored' }
    @{ Operation = 'set'; metadata = 'Ignored' }
    @{ Operation = 'test'; metadata = 'Ignored' }
  ) {
    param($Operation, $metadata, $adapter)

    $yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Class-resource Info
              type: TestClassResource/TestClassResource
              metadata:
                ${metadata}:
                  requireAdapter: $adapter
              properties:
                Name: 'TestClassResource1'
                HashTableProp:
                  Name: 'DSCv3'
                Prop1: foo
"@
    $out = dsc -l trace config $operation -i $yaml 2> $TestDrive/tracing.txt
    $text = $out | Out-String
    $out = $out | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/tracing.txt)
    switch ($Operation) {
      'get' {
        $out.results[0].result.actualState.Name | Should -BeExactly 'TestClassResource1' -Because ("$text`n" + (Get-Content -Raw -Path $TestDrive/tracing.txt))
      }
      'set' {
        $out.results[0].result.beforeState.Name | Should -BeExactly 'TestClassResource1' -Because $text
        $out.results[0].result.afterState.Name | Should -BeExactly 'TestClassResource1' -Because $text
      }
      'test' {
        $out.results[0].result.inDesiredState | Should -BeFalse -Because $text
      }
    }
    if ($metadata -eq 'Microsoft.DSC') {
      "$TestDrive/tracing.txt" | Should -FileContentMatch "Invoking $Operation for '$adapter'" -Because (Get-Content -Raw -Path $TestDrive/tracing.txt)
    }
    if ($adapter -eq 'Microsoft.DSC/PowerShell') {
      (Get-Content -Raw -Path $TestDrive/tracing.txt) | Should -Match "Resource 'Microsoft.DSC/PowerShell' is deprecated" -Because (Get-Content -Raw -Path $TestDrive/tracing.txt)
    }
  }

  It 'Config works with credential object' {
    $yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Class-resource Info
          type: TestClassResource/TestClassResource
          properties:
            Name: 'TestClassResource'
            Credential:
              UserName: 'User'
              Password: 'Password'
"@
    $out = dsc config get -i $yaml | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results.result.actualstate.Credential.UserName | Should -Be 'User'
    $out.results.result.actualState.result.Credential.Password.Length | Should -Not -BeNullOrEmpty
  }

  It 'Config does not work when credential properties are missing required fields' {
    $yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Class-resource credential info
          type: TestClassResource/TestClassResource
          properties:
            Name: 'TestClassResource'
            Credential:
              UserName: 'User'
              OtherProperty: 'Password'
"@
    $out = dsc config get -i $yaml 2>&1 | Out-String
    $LASTEXITCODE | Should -Be 2
    $out | Should -Not -BeNullOrEmpty
    $out | Should -BeLike "*ERROR*Credential object 'Credential' requires both 'username' and 'password' properties*"
  }

  It 'Config get is able to return proper enum value' {
    $yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Class-resource Info
          type: TestClassResource/TestClassResource
          properties:
            Name: 'TestClassResource'
            Ensure: 'Present'
"@

    $out = dsc config get -i $yaml | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results.result.actualState.Ensure | Should -Be 'Present'
  }

  It 'Config export is able to return proper enum value' {
    $yaml = @"
      `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
      resources:
      - name: Working with class-based resources
        type: Microsoft.DSC/PowerShell
        properties:
          resources:
          - name: Class-resource Info
            type: TestClassResource/TestClassResource
"@

    $out = dsc config export -i $yaml | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.resources[0].properties.result.count | Should -Be 5
    $out.resources[0].properties.result[0].Name | Should -Be "Object1"
    $out.resources[0].properties.result[0].Prop1 | Should -Be "Property of object1"
  }

  It 'Expressions get passed correctly to adapted resource' {
    $yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Class-resource Info
          type: TestClassResource/TestClassResource
          properties:
            Name: EchoBack
            Prop1: "[[this is a string literal]"
            EnumProp: 'Expected'
"@
    $out = dsc config get -i $yaml | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results.result.actualState.Name | Should -BeExactly 'EchoBack'
    $out.results.result.actualState.Prop1 | Should -BeExactly '[this is a string literal]'
    $out.results.result.actualState.EnumProp | Should -BeExactly 'Expected'
  }
}

