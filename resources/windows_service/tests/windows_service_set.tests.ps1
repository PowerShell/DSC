# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Service set tests' {
    BeforeDiscovery {
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/Service'
        # Use the Print Spooler service for set tests — it exists on all Windows
        # machines and is safe to reconfigure briefly.
        $testServiceName = 'Spooler'

        function Get-ServiceState {
            param([string]$Name)
            $json = @{ name = $Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            return ($out | ConvertFrom-Json).actualState
        }
    }

    Context 'Input validation' -Skip:(!$IsWindows -or !$isAdmin) {
        It 'Fails when name is not provided' {
            $json = @{ startType = 'Manual' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when input JSON is invalid' {
            $out = 'not-json' | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Set startType' -Skip:(!$IsWindows -or !$isAdmin) {
        BeforeAll {
            $script:originalState = Get-ServiceState -Name $testServiceName
        }

        AfterAll {
            # Restore original start type
            if ($script:originalState) {
                $restoreJson = @{
                    name      = $testServiceName
                    startType = $script:originalState.startType
                } | ConvertTo-Json -Compress
                $restoreJson | dsc resource set -r $resourceType -f - 2>$null
            }
        }

        It 'Can set startType to Disabled' {
            $json = @{ name = $testServiceName; startType = 'Disabled' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.name | Should -BeExactly $testServiceName
            $result.startType | Should -BeExactly 'Disabled'
        }

        It 'Can set startType to Manual' {
            $json = @{ name = $testServiceName; startType = 'Manual' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'Manual'
        }

        It 'Can set startType to Automatic' {
            $json = @{ name = $testServiceName; startType = 'Automatic' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'Automatic'
        }

        It 'Can set startType to AutomaticDelayedStart' {
            $json = @{ name = $testServiceName; startType = 'AutomaticDelayedStart' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'AutomaticDelayedStart'
        }
    }

    Context 'Set description' -Skip:(!$IsWindows -or !$isAdmin) {
        BeforeAll {
            $script:originalState = Get-ServiceState -Name $testServiceName
        }

        AfterAll {
            # Restore original description
            if ($script:originalState -and $script:originalState.description) {
                $restoreJson = @{
                    name        = $testServiceName
                    description = $script:originalState.description
                } | ConvertTo-Json -Compress
                $restoreJson | dsc resource set -r $resourceType -f - 2>$null
            }
        }

        It 'Can set description' {
            $testDesc = 'DSC test description'
            $json = @{ name = $testServiceName; description = $testDesc } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.description | Should -BeExactly $testDesc
        }
    }

    Context 'Set service status' -Skip:(!$IsWindows -or !$isAdmin) {
        BeforeAll {
            $script:originalState = Get-ServiceState -Name $testServiceName
            # Ensure the service is startable (not disabled)
            if ($script:originalState.startType -eq 'Disabled') {
                $json = @{ name = $testServiceName; startType = 'Manual' } | ConvertTo-Json -Compress
                $json | dsc resource set -r $resourceType -f - 2>$null
            }
        }

        AfterAll {
            # Restore original status and start type
            if ($script:originalState) {
                $restoreJson = @{
                    name      = $testServiceName
                    startType = $script:originalState.startType
                    status    = $script:originalState.status
                } | ConvertTo-Json -Compress
                $restoreJson | dsc resource set -r $resourceType -f - 2>$null
            }
        }

        It 'Can start a stopped service' {
            # First ensure it is stopped
            $json = @{ name = $testServiceName; status = 'Stopped' } | ConvertTo-Json -Compress
            $json | dsc resource set -r $resourceType -f - 2>$null

            # Now start it
            $json = @{ name = $testServiceName; status = 'Running' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.status | Should -BeExactly 'Running'
        }

        It 'Can stop a running service' {
            # First ensure it is running
            $json = @{ name = $testServiceName; status = 'Running' } | ConvertTo-Json -Compress
            $json | dsc resource set -r $resourceType -f - 2>$null

            # Now stop it
            $json = @{ name = $testServiceName; status = 'Stopped' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.status | Should -BeExactly 'Stopped'
        }

        It 'Is idempotent when service is already in desired status' {
            # Ensure it is stopped
            $json = @{ name = $testServiceName; status = 'Stopped' } | ConvertTo-Json -Compress
            $json | dsc resource set -r $resourceType -f - 2>$null

            # Set to Stopped again — should succeed without error
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.status | Should -BeExactly 'Stopped'
        }
    }

    Context 'Set multiple properties at once' -Skip:(!$IsWindows -or !$isAdmin) {
        BeforeAll {
            $script:originalState = Get-ServiceState -Name $testServiceName
        }

        AfterAll {
            # Restore original state
            if ($script:originalState) {
                $restoreJson = @{
                    name        = $testServiceName
                    startType   = $script:originalState.startType
                    description = $script:originalState.description
                    status      = $script:originalState.status
                } | ConvertTo-Json -Compress
                $restoreJson | dsc resource set -r $resourceType -f - 2>$null
            }
        }

        It 'Can set startType and description together' {
            $json = @{
                name        = $testServiceName
                startType   = 'Manual'
                description = 'DSC combined test'
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'Manual'
            $result.description | Should -BeExactly 'DSC combined test'
        }

        It 'Returns full service state after set' {
            $json = @{ name = $testServiceName; startType = 'Manual' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.name | Should -Not -BeNullOrEmpty
            $result.displayName | Should -Not -BeNullOrEmpty
            $result._exist | Should -BeTrue
            $result.status | Should -Not -BeNullOrEmpty
            $result.startType | Should -Not -BeNullOrEmpty
            $result.executablePath | Should -Not -BeNullOrEmpty
            $result.logonAccount | Should -Not -BeNullOrEmpty
            $result.errorControl | Should -Not -BeNullOrEmpty
        }
    }

    Context 'Set with no changes (idempotent)' -Skip:(!$IsWindows -or !$isAdmin) {
        It 'Succeeds when only name is provided (no properties to change)' {
            $json = @{ name = $testServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$null
            $LASTEXITCODE | Should -Be 0
            $result = ($out | ConvertFrom-Json).afterState
            $result.name | Should -BeExactly $testServiceName
            $result._exist | Should -BeTrue
        }
    }
}
