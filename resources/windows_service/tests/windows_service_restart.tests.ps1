# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $isAdmin = if ($IsWindows) {
        $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [Security.Principal.WindowsPrincipal]$identity
        $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    } else {
        $false
    }
}

Describe 'Windows Service restart action tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $actionType = 'Microsoft.Windows.Action/RestartService'
        $resourceType = 'Microsoft.Windows/Service'
        $testServiceName = 'Spooler'

        function New-RestartConfiguration {
            param([hashtable]$Properties = @{ name = $testServiceName })

            @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
                resources = @(
                    @{
                        name = 'Restart Print Spooler'
                        type = $actionType
                        properties = $Properties
                    }
                )
            } | ConvertTo-Json -Depth 5 -Compress
        }

        function Set-SpoolerState {
            param([hashtable]$Properties)

            $json = @{ name = $testServiceName } + $Properties | ConvertTo-Json -Compress
            $null = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        }

        function Get-SpoolerProcess {
            Get-CimInstance -ClassName Win32_Service -Filter "Name = '$testServiceName'" -ErrorAction Stop
        }
    }

    It 'Discovers both the service resource and restart action from the manifest list' {
        $resources = dsc resource list $resourceType 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        @($resources) | Should -HaveCount 1
        $resources.type | Should -BeExactly $resourceType

        $actions = dsc action list $actionType 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        @($actions) | Should -HaveCount 1
        $actions.type | Should -BeExactly $actionType
        $actions.supportedOperations | Should -Be @('set')
        $actions.manifest.invoke.requireSecurityContext | Should -BeExactly 'elevated'
        $actions.manifest.invoke.args.whatIfArg | Should -Contain '--what-if'
    }

    It 'Rejects invalid restart input: <case>' -ForEach @(
        @{ case = 'invalid JSON'; json = 'not-json'; exitCode = 2 }
        @{ case = 'missing name'; json = '{}'; exitCode = 2 }
        @{ case = 'null name'; json = '{"name":null}'; exitCode = 2 }
        @{ case = 'unknown property'; json = '{"name":"Spooler","status":"Stopped"}'; exitCode = 2 }
        @{ case = 'empty name'; json = '{"name":""}'; exitCode = 3 }
        @{ case = 'whitespace name'; json = '{"name":"   "}'; exitCode = 3 }
        @{ case = 'null character'; json = '{"name":"Spooler\u0000other"}'; exitCode = 3 }
    ) {
        $out = windows_service restart -w --input $json 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be $exitCode -Because (Get-Content -Raw $TestDrive/error.log)
        $out | Should -BeNullOrEmpty
        (Get-Content -Raw $TestDrive/error.log | ConvertFrom-Json).error | Should -Not -BeNullOrEmpty
    }

    It 'Requires input for restart' {
        $out = windows_service restart 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 1 -Because (Get-Content -Raw $TestDrive/error.log)
        $out | Should -BeNullOrEmpty
        (Get-Content -Raw $TestDrive/error.log | ConvertFrom-Json).error | Should -Match 'Missing --input'
    }

    It 'Fails to restart a nonexistent service' {
        $out = windows_service restart -w --input '{"name":"nonexistent_service_xyz"}' 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 3 -Because (Get-Content -Raw $TestDrive/error.log)
        $out | Should -BeNullOrEmpty
        (Get-Content -Raw $TestDrive/error.log | ConvertFrom-Json).error | Should -Match 'Failed to open service'
    }

    It 'Previews restarting Spooler without changing its state or process' {
        $before = Get-SpoolerProcess
        $json = @{ name = $testServiceName } | ConvertTo-Json -Compress
        $result = windows_service restart -w --input $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result.name | Should -BeExactly $testServiceName
        $result.status | Should -BeExactly 'Running'
        $result._metadata.whatIf | Should -Contain "Would restart service '$testServiceName'"

        $after = Get-SpoolerProcess
        $after.State | Should -BeExactly $before.State
        $after.StartMode | Should -BeExactly $before.StartMode
        $after.ProcessId | Should -Be $before.ProcessId
    }

    Context 'Elevated action invocation' -Skip:(!$isAdmin) {
        BeforeEach {
            $originalState = $null
            $json = @{ name = $testServiceName } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $originalState = ($out | ConvertFrom-Json).actualState
            if ($originalState.startType -eq 'Disabled') {
                Set-SpoolerState -Properties @{ startType = 'Manual' }
            }
        }

        AfterEach {
            if ($originalState) {
                try {
                    Set-SpoolerState -Properties @{ startType = 'Manual' }
                    Set-SpoolerState -Properties @{ status = $originalState.status }
                } finally {
                    Set-SpoolerState -Properties @{ startType = $originalState.startType }
                }
            }
        }

        It 'Restarts an already running Spooler on every invocation' {
            Set-SpoolerState -Properties @{ status = 'Running' }
            foreach ($iteration in 1..2) {
                $before = Get-SpoolerProcess
                $before.ProcessId | Should -BeGreaterThan 0
                $out = New-RestartConfiguration | dsc config set -f - 2>$TestDrive/error.log
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
                $output = $out | ConvertFrom-Json
                $output.hadErrors | Should -BeFalse
                $output.results[0].result.afterState.name | Should -BeExactly $testServiceName
                $output.results[0].result.afterState.status | Should -BeExactly 'Running'

                $after = Get-SpoolerProcess
                $after.State | Should -BeExactly 'Running'
                $after.ProcessId | Should -BeGreaterThan 0
                $after.ProcessId | Should -Not -Be $before.ProcessId
                $after.StartMode | Should -BeExactly $before.StartMode
            }
        }

        It 'Starts a stopped Spooler' {
            Set-SpoolerState -Properties @{ status = 'Stopped' }
            $out = New-RestartConfiguration | dsc config set -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $output = $out | ConvertFrom-Json
            $output.hadErrors | Should -BeFalse
            $output.results[0].result.afterState.status | Should -BeExactly 'Running'
            (Get-Service -Name $testServiceName).Status | Should -Be 'Running'
        }

        It 'Previews the action through DSC without restarting Spooler' {
            Set-SpoolerState -Properties @{ status = 'Running' }
            $before = Get-SpoolerProcess
            $out = New-RestartConfiguration | dsc config set --what-if -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
            $output = $out | ConvertFrom-Json
            $output.hadErrors | Should -BeFalse
            $output.results[0].metadata.whatIf |
                Should -Contain "Would restart service '$testServiceName'"
            $after = Get-SpoolerProcess
            $after.State | Should -BeExactly $before.State
            $after.ProcessId | Should -Be $before.ProcessId
        }

        It 'Rejects invalid action properties through DSC' -ForEach @(
            @{ properties = @{} }
            @{ properties = @{ name = '' } }
            @{ properties = @{ displayName = 'Print Spooler' } }
            @{ properties = @{ name = 'Spooler'; status = 'Stopped' } }
        ) {
            $out = New-RestartConfiguration -Properties $properties |
                dsc config set -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $TestDrive/error.log | Should -Match 'Schema'
        }

        It 'Reports an error when Spooler is disabled' {
            Set-SpoolerState -Properties @{ status = 'Stopped' }
            Set-SpoolerState -Properties @{ startType = 'Disabled' }
            $out = New-RestartConfiguration | dsc config set -f - 2>$TestDrive/error.log
            $LASTEXITCODE | Should -Not -Be 0
            Get-Content -Raw $TestDrive/error.log | Should -Match 'Failed to start service'
            (Get-Service -Name $testServiceName).Status | Should -Be 'Stopped'
        }
    }
}
