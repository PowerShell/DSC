# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Service set tests' -Skip:(!$IsWindows) {
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
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }
    }

    Context 'Input validation' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-ServiceState -Name $testServiceName
        }

        AfterAll {
            # Restore original logon account
            if ($script:originalState -and $script:originalState.logonAccount) {
                $restoreJson = @{
                    name         = $testServiceName
                    logonAccount = $script:originalState.logonAccount
                } | ConvertTo-Json -Compress
                $restoreJson | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            }
        }

        It 'Fails when name is not provided' {
            $json = @{ startType = 'Manual' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when input JSON is invalid' {
            $out = 'not-json' | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when logonAccount is not a built-in service account' {
            $json = @{ name = 'Spooler'; logonAccount = 'DOMAIN\SomeUser' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Accepts LocalSystem as logonAccount' {
            $json = @{ name = 'Spooler'; logonAccount = 'LocalSystem' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        }

        It 'Accepts NT AUTHORITY\NetworkService as logonAccount' {
            $json = @{ name = 'Spooler'; logonAccount = 'NT AUTHORITY\NetworkService' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        }
    }

    Context 'Set startType' -Skip:(!$isAdmin) {
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
                $restoreJson | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            }
        }

        It 'Can set startType to Disabled' {
            $json = @{ name = $testServiceName; startType = 'Disabled' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.name | Should -BeExactly $testServiceName
            $result.startType | Should -BeExactly 'Disabled'
        }

        It 'Can set startType to Manual' {
            $json = @{ name = $testServiceName; startType = 'Manual' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'Manual'
        }

        It 'Can set startType to Automatic' {
            $json = @{ name = $testServiceName; startType = 'Automatic' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'Automatic'
        }

        It 'Can set startType to AutomaticDelayedStart' {
            $json = @{ name = $testServiceName; startType = 'AutomaticDelayedStart' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'AutomaticDelayedStart'
        }
    }

    Context 'Set description' -Skip:(!$isAdmin) {
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
                $restoreJson | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            }
        }

        It 'Can set description' {
            $testDesc = 'DSC test description'
            $json = @{ name = $testServiceName; description = $testDesc } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.description | Should -BeExactly $testDesc
        }
    }

    Context 'Set service status' -Skip:(!$isAdmin) {
        BeforeAll {
            $script:originalState = Get-ServiceState -Name $testServiceName
            # Ensure the service is startable (not disabled)
            if ($script:originalState.startType -eq 'Disabled') {
                $json = @{ name = $testServiceName; startType = 'Manual' } | ConvertTo-Json -Compress
                $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
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
                $restoreJson | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            }
        }

        It 'Can start a stopped service' {
            # First ensure it is stopped
            $json = @{ name = $testServiceName; status = 'Stopped' } | ConvertTo-Json -Compress
            $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log

            # Now start it
            $json = @{ name = $testServiceName; status = 'Running' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.status | Should -BeExactly 'Running'
        }

        It 'Can stop a running service' {
            # First ensure it is running
            $json = @{ name = $testServiceName; status = 'Running' } | ConvertTo-Json -Compress
            $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log

            # Now stop it
            $json = @{ name = $testServiceName; status = 'Stopped' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.status | Should -BeExactly 'Stopped'
        }

        It 'Is idempotent when service is already in desired status' {
            # Ensure it is stopped
            $json = @{ name = $testServiceName; status = 'Stopped' } | ConvertTo-Json -Compress
            $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log

            # Set to Stopped again — should succeed without error
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.status | Should -BeExactly 'Stopped'
        }
    }

    Context 'Set multiple properties at once' -Skip:(!$isAdmin) {
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
                $restoreJson | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            }
        }

        It 'Can set startType and description together' {
            $json = @{
                name        = $testServiceName
                startType   = 'Manual'
                description = 'DSC combined test'
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.startType | Should -BeExactly 'Manual'
            $result.description | Should -BeExactly 'DSC combined test'
        }

        It 'Returns full service state after set' {
            $json = @{ name = $testServiceName; startType = 'Manual' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
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

    Context 'Set with no changes (idempotent)' -Skip:(!$isAdmin) {
        It 'Succeeds when only name is provided (no properties to change)' {
            $json = @{ name = $testServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.name | Should -BeExactly $testServiceName
            $result._exist | Should -BeTrue
        }
    }

    Context 'What-if set' -Skip:(!$isAdmin) {
        It 'Projects a startType change without modifying the service' {
            $before = Get-ServiceState -Name $testServiceName
            $desiredStartType = if ($before.startType -eq 'Disabled') { 'Manual' } else { 'Disabled' }
            $json = @{ name = $testServiceName; startType = $desiredStartType } | ConvertTo-Json -Compress

            $out = $json | dsc resource set --what-if -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $out | ConvertFrom-Json
            $after = $result.afterState

            $after.name | Should -BeExactly $testServiceName
            $after.startType | Should -BeExactly $desiredStartType
            $after._exist | Should -BeTrue
            $after._metadata.whatIf | Should -Not -BeNullOrEmpty
            $after._metadata.whatIf | Should -Contain "Would change startType from '$($before.startType)' to '$desiredStartType'"

            $current = Get-ServiceState -Name $testServiceName
            $current.startType | Should -BeExactly $before.startType
        }

        It 'Projects a displayName change without modifying the service' {
            $before = Get-ServiceState -Name $testServiceName
            $desiredDisplayName = "$($before.displayName) (whatif)"
            $json = @{ name = $testServiceName; displayName = $desiredDisplayName } | ConvertTo-Json -Compress

            $out = $json | dsc resource set --what-if -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $out | ConvertFrom-Json
            $after = $result.afterState

            $after.name | Should -BeExactly $testServiceName
            $after.displayName | Should -BeExactly $desiredDisplayName
            $after._exist | Should -BeTrue
            $after._metadata.whatIf | Should -Not -BeNullOrEmpty
            $after._metadata.whatIf | Should -Contain "Would change displayName from '$($before.displayName)' to '$desiredDisplayName'"

            $current = Get-ServiceState -Name $testServiceName
            $current.displayName | Should -BeExactly $before.displayName
        }

        It 'Projects deletion when _exist is false without modifying the service' {
            $before = Get-ServiceState -Name $testServiceName
            $json = @{ name = $testServiceName; _exist = $false } | ConvertTo-Json -Compress

            $out = $json | dsc resource set --what-if -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = $out | ConvertFrom-Json
            $after = $result.afterState

            $after.name | Should -BeExactly $testServiceName
            $after._exist | Should -BeFalse
            $after._metadata.whatIf | Should -Not -BeNullOrEmpty
            $after._metadata.whatIf | Should -Contain "Would delete service '$testServiceName'"

            $current = Get-ServiceState -Name $testServiceName
            $current._exist | Should -BeTrue
            $current.startType | Should -BeExactly $before.startType
        }
    }
}
