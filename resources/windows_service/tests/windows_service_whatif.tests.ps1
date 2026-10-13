# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'windows service whatif tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $testServiceName = 'Spooler'

        function Get-ServiceState {
            param([string]$Name = $testServiceName)
            $json = @{ name = $Name } | ConvertTo-Json -Compress
            $out = $json | windows_service get --input $json 2>$null | ConvertFrom-Json
            return $out
        }
    }

    It 'Can whatif a startType change' {
        $before = Get-ServiceState
        $desiredStartType = if ($before.startType -eq 'Disabled') { 'Manual' } else { 'Disabled' }
        $json = @{ name = $testServiceName; startType = $desiredStartType } | ConvertTo-Json -Compress

        $result = windows_service set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.name      | Should -BeExactly $testServiceName
        $result.startType | Should -BeExactly $desiredStartType
        $result._exist    | Should -BeTrue
        $result._metadata.whatIf | Should -Not -BeNullOrEmpty
        $result._metadata.whatIf | Should -Contain "Would change startType from '$($before.startType)' to '$desiredStartType'"

        # Assert no mutation happened
        $after = Get-ServiceState
        $after.startType | Should -BeExactly $before.startType
    }

    It 'Can whatif a displayName change' {
        $before = Get-ServiceState
        $desiredDisplayName = "$($before.displayName) (whatif)"
        $json = @{ name = $testServiceName; displayName = $desiredDisplayName } | ConvertTo-Json -Compress

        $result = windows_service set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.name        | Should -BeExactly $testServiceName
        $result.displayName | Should -BeExactly $desiredDisplayName
        $result._exist      | Should -BeTrue
        $result._metadata.whatIf | Should -Not -BeNullOrEmpty
        $result._metadata.whatIf | Should -Contain "Would change displayName from '$($before.displayName)' to '$desiredDisplayName'"

        # Assert no mutation happened
        $after = Get-ServiceState
        $after.displayName | Should -BeExactly $before.displayName
    }

    It 'Can whatif a description change' {
        $before = Get-ServiceState
        $desiredDescription = 'DSC whatif test description'
        $json = @{ name = $testServiceName; description = $desiredDescription } | ConvertTo-Json -Compress

        $result = windows_service set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.name        | Should -BeExactly $testServiceName
        $result.description | Should -BeExactly $desiredDescription
        $result._exist      | Should -BeTrue
        $result._metadata.whatIf | Should -Not -BeNullOrEmpty
        $result._metadata.whatIf | Should -Match "Would change description"

        # Assert no mutation happened
        $after = Get-ServiceState
        $after.description | Should -BeExactly $before.description
    }

    It 'Returns no whatIf messages when no properties would change' {
        $before = Get-ServiceState
        # Pass back the exact current values — nothing should change
        $json = @{
            name      = $testServiceName
            startType = $before.startType
        } | ConvertTo-Json -Compress

        $result = windows_service set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.name      | Should -BeExactly $testServiceName
        $result._exist    | Should -BeTrue
        $result._metadata | Should -BeNullOrEmpty
    }

    It 'Can whatif delete an existing service using _exist is false' {
        $before = Get-ServiceState
        $json = @{ name = $testServiceName; '_exist' = $false } | ConvertTo-Json -Compress

        $result = windows_service set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result.name   | Should -BeExactly $testServiceName
        $result._exist | Should -BeFalse
        $result._metadata.whatIf | Should -Not -BeNullOrEmpty
        $result._metadata.whatIf | Should -Contain "Would delete service '$testServiceName'"

        # Assert no mutation happened — service still exists
        $after = Get-ServiceState
        $after._exist    | Should -BeTrue
        $after.startType | Should -BeExactly $before.startType
    }

    It 'Reports service not found when service does not exist' {
        $json = @{ name = 'DSC-NonExistent-Service-12345' } | ConvertTo-Json -Compress

        $result = windows_service set -w --input $json 2>$null | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        $result._exist | Should -BeFalse
        $result._metadata.whatIf | Should -Not -BeNullOrEmpty
    }
}
