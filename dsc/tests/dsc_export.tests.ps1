# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'resource export tests' {
    
    It 'Export can be called on individual resource' {

        $processes = dsc resource export -r Microsoft/Process
        $LASTEXITCODE | Should -Be 0
        $processes.count | Should -BeGreaterThan 1
    }

    It 'Export can be called on a configuration' {

        $json = @'
            $schema: https://schemas.microsoft.com/dsc/2023/03/configuration.schema.json
            resources:
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
'@
        $out = $json | dsc config export
        $LASTEXITCODE | Should -Be 0
        $config_with_process_list = $out | ConvertFrom-Json
        $config_with_process_list.resources.count | Should -BeGreaterThan 1
    }

    It 'Configuration Export can be piped to configuration Set' {

        $json = @'
            $schema: https://schemas.microsoft.com/dsc/2023/03/configuration.schema.json
            resources:
            - name: Processes
              type: Microsoft/Process
              properties:
                pid: 0
'@
        $out = $json | dsc config export | dsc config set
        $LASTEXITCODE | Should -Be 0
        $set_results = $out | ConvertFrom-Json
        $set_results.results.count | Should -BeGreaterThan 1
        $set_results.results[0].result.afterState.result | Should -BeExactly "Ok"
    }
}
