# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config schema tests' {
    It 'return resource schema' -Skip:(!$IsWindows) {
        $schema = dsc resource schema -r *registry
        $LASTEXITCODE | Should -Be 0
        $schema | Should -Not -BeNullOrEmpty
        $schema = $schema | ConvertFrom-Json
        $schema.'$schema' | Should -BeExactly 'http://json-schema.org/draft-07/schema#'
    }

    It 'return dsc schema: <type>' -Skip:(!$IsWindows) -TestCases @(
        @{ type = 'get-result' }
        @{ type = 'set-result' }
        @{ type = 'test-result' }
        @{ type = 'resource-manifest' }
        @{ type = 'configuration' }
        @{ type = 'configuration-get-result' }
        @{ type = 'configuration-set-result' }
        @{ type = 'configuration-test-result' }
    ) {
        param($type)

        $schema = dsc schema -t $type
        $LASTEXITCODE | Should -Be 0
        $schema | Should -Not -BeNullOrEmpty
        $schema = $schema | ConvertFrom-Json
        $schema.'$schema' | Should -BeExactly 'http://json-schema.org/draft-07/schema#'
    }
}