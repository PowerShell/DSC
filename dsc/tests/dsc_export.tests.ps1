# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'resource export tests' {

    It 'Export can be called on individual resource' {

        $out = dsc resource export -r Microsoft/Process
        $LASTEXITCODE | Should -Be 0
        $config_with_process_list = $out | ConvertFrom-Json
        $config_with_process_list.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
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
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
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
        $config_with_process_list.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
        $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
        $config_with_process_list.resources.count | Should -BeGreaterThan 1
        $config_with_process_list.metadata.'Microsoft.DSC'.operation | Should -BeExactly 'Export'
        # contentVersion on export is always 1.0.0
        $config_with_process_list.contentVersion | Should -BeExactly '1.0.0'
    }

    It 'Configuration Export can be piped to configuration Set' -Skip:(!$IsWindows) {

        $yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
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
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
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
        $out = $yaml | dsc config export -f - 2>&1
        $LASTEXITCODE | Should -Be 0
    }

    It 'Export can be called on individual resource with the use of --output-format as a subcommand' {

      $out = dsc resource export -r Microsoft/Process -o pretty-json
      $LASTEXITCODE | Should -Be 0
      $config_with_process_list = $out | ConvertFrom-Json
      $config_with_process_list.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
      $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
      $config_with_process_list.resources.count | Should -BeGreaterThan 1
    }

    It 'Export can be called on a configuration with the use of --output-format as a subcommand' {

      $yaml = @'
          $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
          resources:
          - name: Processes
            type: Microsoft/Process
            properties:
              pid: 0
'@
      $out = $yaml | dsc config export -o pretty-json -f -
      $LASTEXITCODE | Should -Be 0
      $config_with_process_list = $out | ConvertFrom-Json
      $config_with_process_list.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json'
      $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
      $config_with_process_list.resources.count | Should -BeGreaterThan 1
    }
}
