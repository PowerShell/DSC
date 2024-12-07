# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for listing resources' {
    It 'dsc resource list' {
        $resources = dsc resource list | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $resources | Should -Not -BeNullOrEmpty
        $resources.Count | Should -BeGreaterThan 0
        $resources.type | Should -Contain 'Microsoft.DSC/Assertion'
        $resources.type | Should -Contain 'Microsoft.DSC/Group'
        $resources.type | Should -Contain 'Microsoft/OSInfo'
        ($resources | Where-Object { $_.type -eq 'Microsoft.DSC/Group' }).Kind | Should -BeExactly 'Group'
        ($resources | Where-Object { $_.type -eq 'Microsoft/OSInfo' }).Kind | Should -BeExactly 'Resource'
        ($resources | Where-Object { $_.type -eq 'Microsoft.DSC/PowerShell' }).Kind | Should -BeExactly 'Adapter'
    }

    It 'dsc resource list --tags "<tags>" and --description "<description> work' -TestCases @(
        if ($IsLinux) {
            @{ tags = 'linux'; description = $null; expectedCount = 2; expectedType = @('DSC.PackageManagement/Apt', 'Microsoft/OSInfo') }
        }
        else {
            @{ tags = 'linux'; description = $null; expectedCount = 1; expectedType = 'Microsoft/OSInfo' }
        }
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

    It 'can accept the use of --output-format as a subcommand' {
        $expectedCount = 1
        $expectedType = 'Microsoft/OSInfo'
        $resources = dsc resource list --description "operating system" --output-format pretty-json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resources.Count | Should -Be $expectedCount
        if ($expectedCount -gt 0) {
            $resources.type | Should -BeExactly $expectedType
        }
    }

    It 'Capabilities are returned' {
        $resource = dsc resource list Microsoft/OSInfo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resource.capabilities.Count | Should -Be 2
        $resource.capabilities | Should -Contain 'Get'
        $resource.capabilities | Should -Contain 'Export'
    }

    It 'Invalid adapter returns an error' {
        $out = dsc resource list --adapter 'foo*' 2>&1 | Out-String
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeLike "*ERROR*Adapter 'foo`*' not found*"
    }
}
