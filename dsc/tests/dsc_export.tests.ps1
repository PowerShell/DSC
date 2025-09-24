# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'resource export tests' {
    BeforeAll {
        $env:DSC_TRACE_LEVEL = 'error'
    }

    AfterAll {
        $env:DSC_TRACE_LEVEL = $null
    }

    It 'Export can be called on individual resource' {

        $out = dsc resource export -r Microsoft/Process
        $LASTEXITCODE | Should -Be 0
        $config_with_process_list = $out | ConvertFrom-Json
        $config_with_process_list.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
        $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
        $config_with_process_list.resources.count | Should -BeGreaterThan 1
    }

    It 'get --all can be called on individual resource' {

        $out = dsc resource get --all -r Microsoft/Process
        $LASTEXITCODE | Should -Be 0
        $process_list = $out | ConvertFrom-Json
        $process_list.resources.count | Should -BeGreaterThan 1
        $process_list | % {$_.actualState | Should -Not -BeNullOrEmpty}
    }

    It 'Export can be called on a configuration' {

        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            contentVersion: 1.2.3
            resources:
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
'@
        $out = $yaml | dsc config export -f -
        $LASTEXITCODE | Should -Be 0
        $config_with_process_list = $out | ConvertFrom-Json
        $config_with_process_list.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
        $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
        $config_with_process_list.resources.count | Should -BeGreaterThan 1
        $config_with_process_list.metadata.'Microsoft.DSC'.operation | Should -BeExactly 'export'
        # contentVersion on export is always 1.0.0
        $config_with_process_list.contentVersion | Should -BeExactly '1.0.0'
        $config_with_process_list.resources.name | Should -BeLike 'Process-*'
    }

    It 'Configuration Export can be piped to configuration Set' -Skip:(!$IsWindows) {

        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
'@
        $out = $yaml | dsc config export -f - | dsc config set -f -
        $LASTEXITCODE | Should -Be 0
        $set_results = $out | ConvertFrom-Json
        $set_results.results.count | Should -BeGreaterThan 1
    }

    It 'Duplicate resource types in Configuration Export should not result in error' {

        $yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
'@
        $null = $yaml | dsc config export -f - 2>&1
        $LASTEXITCODE | Should -Be 0
    }

    It 'Export can be called on individual resource with the use of --output-format as a subcommand' {

      $out = dsc resource export -r Microsoft/Process -o pretty-json
      $LASTEXITCODE | Should -Be 0
      $config_with_process_list = $out | ConvertFrom-Json
      $config_with_process_list.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
      $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
      $config_with_process_list.resources.count | Should -BeGreaterThan 1
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
          - name: Processes
            type: Microsoft/Process
            properties:
              pid: 0
'@
      $out = $yaml | dsc config export -o pretty-json -f -
      $LASTEXITCODE | Should -Be 0
      $config_with_process_list = $out | ConvertFrom-Json
      $config_with_process_list.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
      $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
      $config_with_process_list.resources.count | Should -BeGreaterThan 1
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

    It 'Export can surface _kind, _securityContext, and _name from a resource' {
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
        $out.resources[0].kind | Should -BeExactly 'TestKind'
        $out.resources[0].metadata.'Microsoft.DSC'.securityContext | Should -BeExactly 'elevated'
        $out.resources[0].properties.psobject.properties.name | Should -Not -Contain '_kind'
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
