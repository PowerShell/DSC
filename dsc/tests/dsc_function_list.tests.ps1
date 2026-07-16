# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    try {
        $windowWidth = [Console]::WindowWidth
    } catch {
        $consoleUnavailable = $true
    }
}

Describe 'Tests for function list subcommand' {
    It 'Should list all available functions' {
        $out = dsc function list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.count | Should -BeGreaterThan 10
        $out.name | Should -Contain 'reference'
        $out.name | Should -Contain 'envvar'
        $out.description | Should -Not -Contain ''
        $out.category | Should -Not -Contain ''
    }

    It 'Should filter with wildcard' {
        $out = dsc function list 'resource*' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.category | Should -BeExactly 'resource'
        $out.name | Should -BeExactly 'resourceId'
        $out.minArgs | Should -Be 2
        $out.maxArgs | Should -Be 2
        $out.returnTypes | Should -Be @('String')
        $out.description | Should -Not -BeNullOrEmpty
    }

    It 'Table can be not truncated' -Skip:($consoleUnavailable) {
        $output = dsc function list --output-format table-no-truncate
        $LASTEXITCODE | Should -Be 0
        $foundWideLine = $false
        foreach ($line in $output) {
            if ($line.Length -gt $windowWidth) {
                $foundWideLine = $true
            }
        }
        $foundWideLine | Should -BeTrue
    }

    Context 'Category filter' {
        It 'Should filter by a single category' {
            $out = dsc function list --category resource | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
            foreach ($function in $out) {
                $function.category | Should -Contain 'resource'
            }
            $out.name | Should -Contain 'resourceId'
        }

        It 'Should treat multiple categories as an AND filter' {
            $out = dsc function list --category array --category object | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
            foreach ($function in $out) {
                $function.category | Should -Contain 'array'
                $function.category | Should -Contain 'object'
            }
            $out.name | Should -Contain 'tryGet'
        }

        It 'Should match category case-insensitively' {
            $out = dsc function list --category Array | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
            foreach ($function in $out) {
                $function.category | Should -Contain 'array'
            }
        }

        It 'Should error on an invalid category' {
            $out = dsc function list --category notARealCategory 2>&1
            $LASTEXITCODE | Should -Not -Be 0
            ($out | Out-String) | Should -Match 'Invalid function category'
        }
    }

    Context 'Description filter' {
        It 'Should filter on description text' {
            $out = dsc function list --description base64 | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out | Should -Not -BeNullOrEmpty
            foreach ($function in $out) {
                $function.description | Should -Match 'base64'
            }
            $out.name | Should -Contain 'base64'
            $out.name | Should -Contain 'base64ToString'
        }

        It 'Should support wildcards in the description filter' {
            $out = dsc function list --description 'encodes*base64' | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.name | Should -Contain 'base64'
        }

        It 'Should return nothing when the description does not match' {
            $out = dsc function list --description 'zzzNoSuchDescriptionzzz' | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out | Should -BeNullOrEmpty
        }
    }

    Context 'Combined filters' {
        It 'Should combine name, category, and description filters' {
            $out = dsc function list 'try*' --category array --category object --description 'retrieve' | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.name | Should -BeExactly 'tryGet'
        }
    }
}
