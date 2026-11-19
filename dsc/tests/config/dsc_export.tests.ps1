# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'resource export tests' {

    It 'Export can be called on individual resource' {

        $out = dsc resource export -r Test/Export -i '{"count":3}'
        $LASTEXITCODE | Should -Be 0
        $exportedConfig = $out | ConvertFrom-Json
        $exportedConfig.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
        $exportedConfig.resources | Should -HaveCount 3
    }

    It 'get --all can be called on individual resource' {

        $out = dsc resource get --all -r Test/ExportSchemaEmbedded
        $LASTEXITCODE | Should -Be 0
        $instances = $out | ConvertFrom-Json
        $instances | Should -HaveCount 3
        $instances | ForEach-Object {$_.actualState | Should -Not -BeNullOrEmpty}
    }

    It 'Export can be called on a configuration' {

        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            contentVersion: 1.2.3
            resources:
            - name: Instances
              type: Test/Export
              properties:
                count: 3
'@
        $out = $yaml | dsc config export -f -
        $LASTEXITCODE | Should -Be 0
        $exportedConfig = $out | ConvertFrom-Json
        $exportedConfig.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
        $exportedConfig.resources | Should -HaveCount 3
        $exportedConfig.metadata.'Microsoft.DSC'.operation | Should -BeExactly 'export'
        # contentVersion on export is always 1.0.0
        $exportedConfig.contentVersion | Should -BeExactly '1.0.0'
        $exportedConfig.resources.name | Should -BeExactly @('TestName', 'TestName', 'TestName')
    }

    It 'Configuration Export can be piped to configuration Set' {

        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: First instance
              type: Test/Set
            - name: Second instance
              type: Test/WhatIf
'@
        $out = $yaml | dsc config export -f - | dsc config set -f -
        $LASTEXITCODE | Should -Be 0
        $set_results = $out | ConvertFrom-Json
        $set_results.results | Should -HaveCount 2
    }

    It 'Duplicate resource types in Configuration Export should not result in error' {

        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: First instances
              type: Test/Export
              properties:
                count: 3
            - name: Second instances
              type: Test/Export
              properties:
                count: 3
'@
        $null = $yaml | dsc config export -f - 2>&1
        $LASTEXITCODE | Should -Be 0
    }

    It 'Export can be called on individual resource with the use of --output-format as a subcommand' {

      $out = dsc resource export -r Test/Export -i '{"count":3}' -o pretty-json
      $LASTEXITCODE | Should -Be 0
      $exportedConfig = $out | ConvertFrom-Json
      $exportedConfig.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
      $exportedConfig.resources | Should -HaveCount 3
    }

    It 'Export can be called on resource with input' {
        $out = '{"count":3}' | dsc resource export -r Test/Export -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 3
        $out.resources[0].type | Should -BeExactly 'Test/Export'
        $out.resources[0].properties.count | Should -Be 0
        $out.resources[1].type | Should -BeExactly 'Test/Export'
        $out.resources[1].properties.count | Should -Be 1
        $out.resources[2].type | Should -BeExactly 'Test/Export'
        $out.resources[2].properties.count | Should -Be 2
    }

    It 'Export can be called on a configuration with the use of --output-format as a subcommand' {

      $yaml = @'
          $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
          resources:
          - name: Instances
            type: Test/Export
            properties:
              count: 3
'@
      $out = $yaml | dsc config export -o pretty-json -f -
      $LASTEXITCODE | Should -Be 0
      $exportedConfig = $out | ConvertFrom-Json
      $exportedConfig.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
      $exportedConfig.resources | Should -HaveCount 3
    }

    It 'Export for config preserves metadata' {
        $yaml = @'
          $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
          metadata:
            winget:
              processor: dscv3
            hello: world
          resources:
            - name: OS
              type: Microsoft/OSInfo
'@
        $out = $yaml | dsc config export -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.metadata.winget.processor | Should -BeExactly 'dscv3'
        $out.metadata.hello | Should -BeExactly 'world'
        $out.metadata.'Microsoft.DSC'.operation | Should -BeExactly 'export'
    }

    It 'Works with Exporter resource' {
      $yaml = @'
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
resources:
  - name: export this
    type: Test/Exporter
    properties:
      typeNames:
        - Test/Foo
        - Test/Bar
'@
      $out = dsc config export -i $yaml | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.resources | Should -HaveCount 2
      $out.resources[0].type | Should -BeExactly 'Test/Foo'
      $out.resources[0].name | Should -BeExactly 'test'
      $out.resources[0].properties.psobject.properties | Should -HaveCount 2
      $out.resources[0].properties.foo | Should -BeExactly 'bar'
      $out.resources[0].properties.hello | Should -BeExactly 'world'
      $out.resources[1].type | Should -BeExactly 'Test/Bar'
      $out.resources[1].name | Should -BeExactly 'test'
      $out.resources[1].properties.psobject.properties | Should -HaveCount 2
      $out.resources[1].properties.foo | Should -BeExactly 'bar'
      $out.resources[1].properties.hello | Should -BeExactly 'world'
    }

    It 'Export can surface _securityContext and _name from a resource' {
      $yaml = @'
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
resources:
  - name: Test Export
    type: Test/Export
    properties:
      count: 1
'@
        $out = dsc config export -i $yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 1
        $out.resources[0].name | Should -BeExactly 'TestName'
        $out.resources[0].metadata.'Microsoft.DSC'.securityContext | Should -BeExactly 'elevated'
        $out.resources[0].properties.psobject.properties.name | Should -Not -Contain '_securityContext'
        $out.resources[0].properties.psobject.properties.name | Should -Not -Contain '_name'
    }

    It 'Export can be used with a resource that only implements Get with filter' {
      $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: NoExport
    type: Test/Get
    properties:
      name: two
'@
      $out = dsc config export -i $yaml | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.resources.count | Should -Be 1
      $out.resources[0].type | Should -BeExactly 'Test/Get'
      $out.resources[0].properties.name | Should -BeExactly 'two'
      $out.resources[0].properties.id | Should -Be 2
    }

    It 'Export can be used with a resource that only implements Get with no filter' {
      $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: NoFilter
    type: Test/Get
'@
      $out = dsc config export -i $yaml | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.resources.count | Should -Be 1
      $out.resources[0].type | Should -BeExactly 'Test/Get'
      $out.resources[0].properties.name | Should -BeExactly 'one'
      $out.resources[0].properties.id | Should -Be 1
    }
}

Describe 'engine export filtering tests' {
    It 'engine filters properties for a resource that does not support filtering natively' {
        $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: No Native Filtering
  type: Test/ExportSchemaNoFiltering
  properties:
    name: '*e*'
'@
        $out = dsc --trace-level info config export -i $yaml 2>$TESTDRIVE/error.log | ConvertFrom-Json
        $errorlog = Get-Content "$TESTDRIVE/error.log" -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorlog
        @($out.resources).Count | Should -Be 2
        $out.resources.properties.name | Should -Be @('Steve', 'Tess')
        $errorlog | Should -Match 'does not support export filtering, the engine will filter the exported instances'
    }

    It 'INFO message indicates engine export filtering is experimental' {
        $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: No Native Filtering
  type: Test/ExportSchemaNoFiltering
  properties:
    name: 'Gijs'
'@
        $out = dsc --trace-level info config export -i $yaml 2>$TESTDRIVE/error.log | ConvertFrom-Json
        $errorlog = Get-Content "$TESTDRIVE/error.log" -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorlog
        @($out.resources).Count | Should -Be 1
        $out.resources[0].properties.name | Should -BeExactly 'Gijs'
        $errorlog | Should -BeLike "*INFO*Resource 'Test/ExportSchemaNoFiltering' does not support export filtering, the engine will filter the exported instances (experimental feature)*"
    }

    It 'no INFO message is emitted when no filter input is provided' {
        $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: No Native Filtering
  type: Test/ExportSchemaNoFiltering
'@
        $out = dsc --trace-level info config export -i $yaml 2>$TESTDRIVE/error.log | ConvertFrom-Json
        $errorlog = Get-Content "$TESTDRIVE/error.log" -Raw
        $LASTEXITCODE | Should -Be 0 -Because $errorlog
        @($out.resources).Count | Should -Be 3
        $errorlog | Should -Not -Match 'the engine will filter the exported instances'
    }
}
