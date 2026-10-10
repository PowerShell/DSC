# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Test resource executable error handling' {
    It 'Returns failure for invalid <Subcommand> input' -ForEach @(
        @{ Subcommand = 'copy-resource' }
        @{ Subcommand = 'delete' }
        @{ Subcommand = 'exist' }
        @{ Subcommand = 'exit-code' }
        @{ Subcommand = 'export' }
        @{ Subcommand = 'export-schema' }
        @{ Subcommand = 'exporter' }
        @{ Subcommand = 'get' }
        @{ Subcommand = 'in-desired-state' }
        @{ Subcommand = 'metadata' }
        @{ Subcommand = 'operation'; ExtraArguments = @('--operation', 'get') }
        @{ Subcommand = 'refresh-env'; ExtraArguments = @('--operation', 'get') }
        @{ Subcommand = 'restart-required' }
        @{ Subcommand = 'schema-default' }
        @{ Subcommand = 'sleep' }
        @{ Subcommand = 'state-and-diff' }
    ) {
        $arguments = @($Subcommand, '--input', '{invalid') + $ExtraArguments

        $null = & dsctest @arguments 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Error JSON does not match schema'
    }

    It 'Returns failure when an adapter receives invalid input' {
        $null = dsctest adapter `
            --operation set `
            --resource-type Adapted/One `
            --input '{invalid' 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Error adapting resource'
    }

    It 'Returns failure when a named instance does not exist' {
        $null = dsctest get --input '{"name":"missing"}' 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'No instance found with name'
    }

    It 'Returns failure when an ID instance does not exist' {
        $null = dsctest get --input '{"id":999}' 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'No instance found with id'
    }
}

Describe 'Rust resource executable error handling' {
    It 'Returns failure when FileContent has no operation' {
        $null = filecontent 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Missing operation'
    }

    It 'Returns failure when FileContent has no input' {
        $null = filecontent get 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Missing --input argument'
    }

    It 'Returns failure when FileContent input is invalid JSON' {
        $null = filecontent get --input '{invalid' 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 2
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Invalid JSON input'
    }

    It 'Returns failure when DSC Echo input is invalid JSON' {
        $null = dscecho --input '{invalid' 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'JSON does not match schema'
    }

    It 'Returns failure when Process has no operation' {
        $null = process 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
    }

    It 'Returns failure when Process has an unknown operation' {
        $null = process unknown 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
    }

    It 'Returns piped input from the Process test operation' {
        $result = 'hello' | process test 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 0
        $result[0] | Should -BeExactly 'hello'
    }

    It 'Returns failure when OSInfo test input is invalid JSON' {
        $null = '{invalid' | osinfo test 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Not -BeNullOrEmpty
    }

    It 'Returns failure when y2j input is neither JSON nor YAML' {
        $null = "`tinvalid" | y2j 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Input is not valid JSON or YAML'
    }
}
