# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'resource export tests' {
    
    It 'Export can be called on individual resource' {

        $out = dsc resource export -r Microsoft/Process
        $LASTEXITCODE | Should -Be 0
        $config_with_process_list = $out | ConvertFrom-Json
        $config_with_process_list.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json'
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
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
'@
        $out = $yaml | dsc config export
        $LASTEXITCODE | Should -Be 0
        $config_with_process_list = $out | ConvertFrom-Json
        $config_with_process_list.'$schema' | Should -BeExactly 'https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json'
        $config_with_process_list.'resources' | Should -Not -BeNullOrEmpty
        $config_with_process_list.resources.count | Should -BeGreaterThan 1
    }

    It 'Configuration Export can be piped to configuration Set' -Skip:(!$IsWindows) {

        $yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
'@
        $out = $yaml | dsc config export | dsc config set
        $LASTEXITCODE | Should -Be 0
        $set_results = $out | ConvertFrom-Json
        $set_results.results.count | Should -BeGreaterThan 1
    }

    It 'Duplicate resource types in Configuration Export should result in error' {

        $yaml = @'
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
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
        $out = $yaml | dsc config export 2>&1
        $LASTEXITCODE | Should -Be 2
        $out | out-string | Should -BeLike '*specified multiple times*'
    }
}
