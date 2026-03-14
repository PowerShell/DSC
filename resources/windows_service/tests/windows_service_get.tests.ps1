# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Service get tests' -Skip:(!$IsWindows) {
    BeforeAll {
        # Use a well-known Windows service that exists on all Windows machines
        $resourceType = 'Microsoft.Windows/Service'
        $knownServiceName = 'wuauserv'
        $service = Get-Service -Name $knownServiceName -ErrorAction Stop
        $knownDisplayName = $service.DisplayName
    }

    Context 'Get by name' {
        It 'Returns service info for an existing service' {
            $json = @{ name = $knownServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.name | Should -BeExactly $knownServiceName
            $result.displayName | Should -BeExactly $knownDisplayName
            $result._exist | Should -BeTrue
            $result.status | Should -Not -BeNullOrEmpty
            $result.startType | Should -Not -BeNullOrEmpty
            $result.executablePath | Should -Not -BeNullOrEmpty
            $result.logonAccount | Should -Not -BeNullOrEmpty
            $result.errorControl | Should -Not -BeNullOrEmpty
        }

        It 'Returns _exist false for a non-existent service' {
            $json = @{ name = 'nonexistent_service_xyz' } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.name | Should -BeExactly 'nonexistent_service_xyz'
            $result._exist | Should -BeFalse
            $result.PSObject.Properties.Name | Should -Not -Contain 'status'
        }
    }

    Context 'Get by displayName' {
        It 'Returns service info when only displayName is provided' {
            $json = @{ displayName = $knownDisplayName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.name | Should -BeExactly $knownServiceName
            $result.displayName | Should -BeExactly $knownDisplayName
            $result._exist | Should -BeTrue
        }

        It 'Returns _exist false for a non-existent display name' {
            $json = @{ displayName = 'Nonexistent Display Name XYZ' } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result._exist | Should -BeFalse
        }
    }

    Context 'Get by both name and displayName' {
        It 'Returns service info when both name and displayName match' {
            $json = @{ name = $knownServiceName; displayName = $knownDisplayName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.name | Should -BeExactly $knownServiceName
            $result.displayName | Should -BeExactly $knownDisplayName
            $result._exist | Should -BeTrue
        }

        It 'Returns error when name and displayName do not match' {
            $json = @{ name = $knownServiceName; displayName = 'Wrong Display Name' } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Service properties' {
        It 'Returns valid startType values' {
            $json = @{ name = $knownServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.startType | Should -BeIn @('Automatic', 'AutomaticDelayedStart', 'Manual', 'Disabled')
        }

        It 'Returns valid status values' {
            $json = @{ name = $knownServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.status | Should -BeIn @('Running', 'Stopped', 'Paused', 'StartPending', 'StopPending', 'PausePending', 'ContinuePending')
        }

        It 'Returns valid errorControl values' {
            $json = @{ name = $knownServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.errorControl | Should -BeIn @('Ignore', 'Normal', 'Severe', 'Critical')
        }

        It 'Returns dependencies as an array when present' {
            $json = @{ name = $knownServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            if ($null -ne $result.dependencies) {
                $result.dependencies | Should -BeOfType [System.Object]
                $result.dependencies.Count | Should -BeGreaterThan 0
            }
        }
    }
}
