# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for copy loops' {
    It 'Works for resources' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex())]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@
        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 3
        $out.results[0].name | Should -Be 'Test-0'
        $out.results[0].result.actualState.output | Should -Be 'Hello'
        $out.results[1].name | Should -Be 'Test-1'
        $out.results[1].result.actualState.output | Should -Be 'Hello'
        $out.results[2].name | Should -Be 'Test-2'
        $out.results[2].result.actualState.output | Should -Be 'Hello'
    }

    It 'copyIndex() works with offset' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex(10))]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@

        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 3
        $out.results[0].name | Should -Be 'Test-10'
        $out.results[0].result.actualState.output | Should -Be 'Hello'
        $out.results[1].name | Should -Be 'Test-11'
        $out.results[1].result.actualState.output | Should -Be 'Hello'
        $out.results[2].name | Should -Be 'Test-12'
        $out.results[2].result.actualState.output | Should -Be 'Hello'
    }

    It 'copyIndex() with negative index returns error' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex(-1))]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@

        $null = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because ((Get-Content $testdrive/error.log) | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -Match 'The offset cannot be negative'
    }

    It 'Copy works with count 0' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex())]"
  copy:
    name: testLoop
    count: 0
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@

        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 0
    }

    It 'copyIndex() with loop name works' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex('testLoop'))]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@
        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 3
        $out.results[0].name | Should -Be 'Test-0'
        $out.results[0].result.actualState.output | Should -Be 'Hello'
        $out.results[1].name | Should -Be 'Test-1'
        $out.results[1].result.actualState.output | Should -Be 'Hello'
        $out.results[2].name | Should -Be 'Test-2'
        $out.results[2].result.actualState.output | Should -Be 'Hello'
    }

    It 'copyIndex() with invalid loop name "<name>" returns error' -TestCases @(
        @{ name = "'noSuchLoop'" }
        @{ name = "'noSuchLoop', 1" }
    ){
        param($name)
        $configYaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex($name))]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
"@

        $null = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because ((Get-Content $testdrive/error.log) | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -Match "The specified loop name 'noSuchLoop' was not found"
    }

    It 'Copy mode is not supported' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex())]"
  copy:
    name: testLoop
    count: 3
    mode: serial
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@
        $null = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because ((Get-Content $testdrive/error.log) | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -Match "Copy mode is not supported"
    }

    It 'Copy batch size is not supported' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Test-{0}', copyIndex())]"
  copy:
    name: testLoop
    count: 3
    batchSize: 2
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@
        $null = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because ((Get-Content $testdrive/error.log) | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -Match "Copy batch size is not supported"
    }

    It 'Name expression during copy must be a string' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[copyIndex()]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@
        $null = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2 -Because ((Get-Content $testdrive/error.log) | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -Match "Copy name result is not a string"
    }

    It 'Copy works with parameters in resource name' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  prefix:
    type: string
    defaultValue: srv
resources:
- name: "[concat(parameters('prefix'), '-', string(copyIndex()))]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: Hello
'@
        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 3
        $out.results[0].name | Should -Be 'srv-0'
        $out.results[0].result.actualState.output | Should -Be 'Hello'
        $out.results[1].name | Should -Be 'srv-1'
        $out.results[1].result.actualState.output | Should -Be 'Hello'
        $out.results[2].name | Should -Be 'srv-2'
        $out.results[2].result.actualState.output | Should -Be 'Hello'
    }

    It 'Copy works with parameters in properties' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  environment:
    type: string
    defaultValue: test
resources:
- name: "[format('Server-{0}', copyIndex())]"
  copy:
    name: testLoop
    count: 2
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat('Environment: ', parameters('environment'))]"
'@
        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 2
        $out.results[0].name | Should -Be 'Server-0'
        $out.results[0].result.actualState.output | Should -Be 'Environment: test'
        $out.results[1].name | Should -Be 'Server-1'
        $out.results[1].result.actualState.output | Should -Be 'Environment: test'
    }

    It 'Copy works with complex parameter expressions' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  config:
    type: object
    defaultValue:
      prefix: app
      environment: prod
resources:
- name: "[concat(parameters('config').prefix, '-', string(copyIndex()), '-', parameters('config').environment)]"
  copy:
    name: testLoop
    count: 2
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      serviceName: "[concat(parameters('config').prefix, '-service-', string(copyIndex()))]"
      environment: "[parameters('config').environment]"
      index: "[copyIndex()]"
'@
        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 2
        $out.results[0].name | Should -Be 'app-0-prod'
        $out.results[0].result.actualState.output.serviceName | Should -Be 'app-service-0'
        $out.results[0].result.actualState.output.environment | Should -Be 'prod'
        $out.results[0].result.actualState.output.index | Should -Be 0
        $out.results[1].name | Should -Be 'app-1-prod'
        $out.results[1].result.actualState.output.serviceName | Should -Be 'app-service-1'
        $out.results[1].result.actualState.output.environment | Should -Be 'prod'
        $out.results[1].result.actualState.output.index | Should -Be 1
    }

    It 'Copy works with parameters and copyIndex in same expression' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  prefix:
    type: string
    defaultValue: item
  offset:
    type: int
    defaultValue: 100
resources:
- name: "[concat(parameters('prefix'), '-', string(add(copyIndex(), parameters('offset'))))]"
  copy:
    name: testLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat('Item ', string(add(copyIndex(), parameters('offset'))), ' with prefix ', parameters('prefix'))]"
'@
        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because ((Get-Content $testdrive/error.log) | Out-String)
        $out.results.Count | Should -Be 3
        $out.results[0].name | Should -Be 'item-100'
        $out.results[0].result.actualState.output | Should -Be 'Item 100 with prefix item'
        $out.results[1].name | Should -Be 'item-101'
        $out.results[1].result.actualState.output | Should -Be 'Item 101 with prefix item'
        $out.results[2].name | Should -Be 'item-102'
        $out.results[2].result.actualState.output | Should -Be 'Item 102 with prefix item'
    }
}
