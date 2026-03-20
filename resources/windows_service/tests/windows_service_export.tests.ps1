# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Service export tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/Service'

        function Invoke-DscExport {
            param(
                [string]$InputJson
            )
            if ($InputJson) {
                $raw = $InputJson | dsc resource export -r $resourceType -f - 2>$testdrive/error.log
            } else {
                $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
            }
            $parsed = $raw | ConvertFrom-Json
            return $parsed
        }
    }

    Context 'Export without filter' {
        It 'Returns multiple services' {
            $result = Invoke-DscExport
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 10
        }

        It 'Each exported service has required properties' -Skip:(!$IsWindows) {
            $result = Invoke-DscExport
            foreach ($resource in $result.resources | Select-Object -First 5) {
                $svc = $resource.properties
                $svc.name | Should -Not -BeNullOrEmpty
                $svc.displayName | Should -Not -BeNullOrEmpty
                $svc._exist | Should -BeTrue
                $svc.status | Should -Not -BeNullOrEmpty
                $svc.startType | Should -Not -BeNullOrEmpty
                $svc.executablePath | Should -Not -BeNullOrEmpty
                $svc.logonAccount | Should -Not -BeNullOrEmpty
                $svc.errorControl | Should -Not -BeNullOrEmpty
            }
        }

        It 'Sets the correct resource type on each entry' {
            $result = Invoke-DscExport
            foreach ($resource in $result.resources | Select-Object -First 5) {
                $resource.type | Should -BeExactly $resourceType
            }
        }
    }

    Context 'Export with name filter' {
        It 'Filters by exact service name' {
            $json = @{ name = 'wuauserv' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -Be 1
            $result.resources[0].properties.name | Should -BeExactly 'wuauserv'
        }

        It 'Filters by name with leading wildcard' {
            $json = @{ name = '*serv' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.name | Should -BeLike '*serv'
            }
        }

        It 'Filters by name with trailing wildcard' {
            $json = @{ name = 'w*' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.name | Should -BeLike 'w*'
            }
        }

        It 'Filters by name with surrounding wildcards' {
            $json = @{ name = '*update*' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.name | Should -BeLike '*update*'
            }
        }

        It 'Returns empty when name filter matches nothing' {
            $json = @{ name = 'nonexistent_service_xyz_12345' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -Be 0
        }
    }

    Context 'Export with displayName filter' {
        It 'Filters by display name with wildcard' {
            $json = @{ displayName = '*Update*' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.displayName | Should -BeLike '*Update*'
            }
        }

        It 'Filters by exact display name' {
            $service = Get-Service -Name 'wuauserv' -ErrorAction Stop
            $knownDisplayName = $service.DisplayName
            $json = @{ displayName = $knownDisplayName } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -Be 1
            $result.resources[0].properties.displayName | Should -BeExactly $knownDisplayName
            $result.resources[0].properties.name | Should -BeExactly 'wuauserv'
        }
    }

    Context 'Export with status filter' {
        It 'Filters by Running status' {
            $json = @{ status = 'Running' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.status | Should -BeExactly 'Running'
            }
        }

        It 'Filters by Stopped status' {
            $json = @{ status = 'Stopped' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.status | Should -BeExactly 'Stopped'
            }
        }
    }

    Context 'Export with startType filter' {
        It 'Filters by Automatic start type' {
            $json = @{ startType = 'Automatic' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.startType | Should -BeExactly 'Automatic'
            }
        }

        It 'Filters by Manual start type' {
            $json = @{ startType = 'Manual' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.startType | Should -BeExactly 'Manual'
            }
        }

        It 'Filters by Disabled start type' {
            $json = @{ startType = 'Disabled' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterOrEqual 0
            foreach ($resource in $result.resources) {
                $resource.properties.startType | Should -BeExactly 'Disabled'
            }
        }
    }

    Context 'Export with multi-field filter' {
        It 'Filters by status AND startType together' {
            $json = @{ status = 'Running'; startType = 'Automatic' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.status | Should -BeExactly 'Running'
                $resource.properties.startType | Should -BeExactly 'Automatic'
            }
        }

        It 'Filters by name wildcard AND status' {
            $json = @{ name = 'w*'; status = 'Stopped' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            foreach ($resource in $result.resources) {
                $resource.properties.name | Should -BeLike 'w*'
                $resource.properties.status | Should -BeExactly 'Stopped'
            }
        }
    }

    Context 'Export with dependencies filter' {
        It 'Filters by a single dependency' {
            $json = @{ dependencies = @('rpcss') } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.dependencies | Should -Not -BeNullOrEmpty
                ($resource.properties.dependencies | ForEach-Object { $_.ToLower() }) | Should -Contain 'rpcss'
            }
        }
    }

    Context 'Export property validation' {
        It 'All exported services have valid startType values' {
            $json = @{ name = 'w*' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $validStartTypes = @('Automatic', 'AutomaticDelayedStart', 'Manual', 'Disabled')
            foreach ($resource in $result.resources) {
                $resource.properties.startType | Should -BeIn $validStartTypes
            }
        }

        It 'All exported services have valid status values' {
            $json = @{ name = 'w*' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $validStatuses = @('Running', 'Stopped', 'Paused', 'StartPending', 'StopPending', 'PausePending', 'ContinuePending')
            foreach ($resource in $result.resources) {
                $resource.properties.status | Should -BeIn $validStatuses
            }
        }

        It 'All exported services have valid errorControl values' {
            $json = @{ name = 'w*' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $validErrorControls = @('Ignore', 'Normal', 'Severe', 'Critical')
            foreach ($resource in $result.resources) {
                $resource.properties.errorControl | Should -BeIn $validErrorControls
            }
        }

        It 'Dependencies is an array when present' {
            $json = @{ dependencies = @('rpcss') } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            foreach ($resource in $result.resources | Select-Object -First 3) {
                $resource.properties.dependencies | Should -BeOfType [System.Object]
                $resource.properties.dependencies.Count | Should -BeGreaterThan 0
            }
        }
    }
}
