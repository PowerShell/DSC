# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Service export tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/Service'

        function Invoke-DscExport {
            $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
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

    Context 'Export with input' {
        It 'Returns an error when export input is provided' {
            $json = @{ name = 'wuauserv' } | ConvertTo-Json -Compress
            dsc resource export -r $resourceType -i $json 2>$testdrive/error.log | Out-Null
            $LASTEXITCODE | Should -Be 2
            (Get-Content -Raw $testdrive/error.log) | Should -Match 'does not support export filtering'
        }
    }
}
