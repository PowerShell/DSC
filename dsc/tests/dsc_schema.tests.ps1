# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'config schema tests' {
    BeforeDiscovery {
        $out = dsc schema --type 2>&1
        write-verbose -verbose "Output from 'dsc schema --type': $out"
        $isMatch = $out -match '\[possible values: (?<values>.*?)\]'
        if (-not $isMatch) {
            throw "Failed to parse schema types from output: $out"
        }
        write-verbose -Verbose "Regex match success: $isMatch"
        write-verbose -verbose ($matches | Out-String)
        write-verbose -Verbose "Matched schema types: $($matches['values'])"
        $schemaTypes = $matches['values'].Split(',').Trim()
        $schemaTestCases = $schemaTypes | ForEach-Object { @{ type = $_ } }
    }

    It 'return resource schema' -Skip:(!$IsWindows) {
        $schema = dsc resource schema -r Microsoft.Windows/Registry
        $LASTEXITCODE | Should -Be 0
        $schema | Should -Not -BeNullOrEmpty
        $schema = $schema | ConvertFrom-Json
        $schema.'$schema' | Should -BeExactly 'https://json-schema.org/draft/2020-12/schema'
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
        $schema.'$schema' | Should -BeExactly 'https://json-schema.org/draft/2020-12/schema'
    }

    It 'can accept the use of --output-format as a subcommand' {
        $schema = dsc resource schema -r Microsoft.DSC.Debug/Echo -o pretty-json
        $LASTEXITCODE | Should -Be 0
        $schema | Should -Not -BeNullOrEmpty
        $schema = $schema | ConvertFrom-Json
        $schema.'$schema' | Should -BeExactly 'https://json-schema.org/draft/2020-12/schema'
    }

    It 'schema uses camelCase for <type>' -TestCases $schemaTestCases {
        param($type)

        $schema = dsc schema -t $type | ConvertFrom-Json -Depth 20 -AsHashtable
        $LASTEXITCODE | Should -Be 0

        foreach ($property in $schema.properties.keys) {
            $property | Should -MatchExactly '^[$_]?[a-z][a-zA-Z0-9]*$' -Because "Property '$property' does not follow camelCase convention."
        }

        foreach ($def in $schema.'$defs'.keys) {
            if ($null -ne $schema.'$defs'[$def].enum) {
                foreach ($enumValue in $schema.'$defs'[$def].enum) {
                    $enumValue | Should -MatchExactly '^[a-z][a-zA-Z0-9]*$' -Because "Enum value '$enumValue' in definition '$def' does not follow camelCase convention."
                }
            }

            if ($null -ne $schema.'$defs'[$def].properties) {
                foreach ($property in $schema.'$defs'[$def].properties.keys) {
                    if ($property -eq 'Microsoft.DSC') {
                        continue
                    }
                    $property | Should -MatchExactly '^[$_]?[a-z][a-zA-Z0-9]*$' -Because "Property '$property' in definition '$def' does not follow camelCase convention."
                }
            }
        }
    }
}
