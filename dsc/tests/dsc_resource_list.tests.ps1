# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for listing resources' {
    It 'dsc resource list' {
        $resources = dsc resource list | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resources | Should -Not -BeNullOrEmpty
        $resources.Count | Should -BeGreaterThan 0
        $resources.type | Should -Contain 'DSC/AssertionGroup'
        $resources.type | Should -Contain 'DSC/Group'
        $resources.type | Should -Contain 'DSC/ParallelGroup'
        $resources.type | Should -Contain 'Microsoft/OSInfo'
    }

    It 'dsc resource list --tags "<tags>" and --description "<description> work' -TestCases @(
        @{ tags = 'linux'; description = $null; expectedCount = 1; expectedType = 'Microsoft/OSInfo' }
        @{ tags = $null; description = 'operating system'; expectedCount = 1; expectedType = 'Microsoft/OSInfo' }
        @{ tags = 'linux'; description = 'operating system'; expectedCount = 1; expectedType = 'Microsoft/OSInfo' }
        @{ tags = 'notfound'; description = 'operating system'; expectedCount = 0; expectedType = $null }
        @{ tags = 'linux'; description = 'notfound'; expectedCount = 0; expectedType = $null }
        @{ tags = 'notfound'; description = 'notfound'; expectedCount = 0; expectedType = $null }
    ) {
        param($tags, $description, $expectedCount, $expectedType)

        if ($tags -and $description) {
            $resources = dsc resource list --tags $tags --description $description | ConvertFrom-Json
        }
        elseif ($tags) {
            $resources = dsc resource list --tags $tags | ConvertFrom-Json
        }
        else {
            $resources = dsc resource list --description $description | ConvertFrom-Json
        }

        $LASTEXITCODE | Should -Be 0
        $resources.Count | Should -Be $expectedCount
        if ($expectedCount -gt 0) {
            $resources.type | Should -BeExactly $expectedType
        }
    }

    It 'can accept the use of --format as a subcommand' {
        $expectedCount = 1
        $expectedType = 'Microsoft/OSInfo'
        $resources = dsc resource list --description "operating system" --format pretty-json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resources.Count | Should -Be $expectedCount
        if ($expectedCount -gt 0) {
            $resources.type | Should -BeExactly $expectedType
        }
    }
}
