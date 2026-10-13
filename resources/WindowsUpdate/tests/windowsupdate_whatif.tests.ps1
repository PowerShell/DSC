# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update WhatIf operation tests' -Skip:(!$IsWindows) {
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
        $resourceType = 'Microsoft.Windows/UpdateList'
    }

    Context 'Set what-if operation' -Skip:(!$isAdmin -or !$IsWindows) {
        It 'Can whatif installing an update without installing it' {
            # Find an update that is not installed
            $exportOut = '{"updates": [{"isInstalled": false}]}' | dsc resource export -r $resourceType -f - 2>&1

            if ($LASTEXITCODE -ne 0) {
                Set-ItResult -Skipped -Because 'export operation failed'
                return
            }

            $exported = $exportOut | ConvertFrom-Json
            if ($exported.updates.Count -eq 0) {
                Set-ItResult -Skipped -Because 'no uninstalled updates are available'
                return
            }

            $testUpdate = $exported.updates[0]
            $json = @{
                updates = @(
                    @{
                        id = $testUpdate.id
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress

            $out = $json | dsc resource set -r $resourceType -f - -w 2>&1
            $LASTEXITCODE | Should -Be 0

            $result = $out | ConvertFrom-Json
            $result.afterState.updates[0].id | Should -Be $testUpdate.id
            $result.afterState.updates[0].isInstalled | Should -Be $true
            $result.afterState.updates[0]._metadata.whatIf | Should -Not -BeNullOrEmpty
            $result.afterState.updates[0]._metadata.whatIf | Should -Contain "Would install update '$($testUpdate.title)'"

            # Assert no mutation happened
            $getOut = $json | dsc resource get -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Be 0
            ($getOut | ConvertFrom-Json).actualState.updates[0].isInstalled | Should -Be $false
        }

        It 'Returns current state without whatIf messages for an already installed update' {
            # Find an update that is already installed
            $exportOut = '{"updates": [{"isInstalled": true}]}' | dsc resource export -r $resourceType -f - 2>&1

            if ($LASTEXITCODE -ne 0) {
                Set-ItResult -Skipped -Because 'export operation failed'
                return
            }

            $exported = $exportOut | ConvertFrom-Json
            if ($exported.updates.Count -eq 0) {
                Set-ItResult -Skipped -Because 'no installed updates are available'
                return
            }

            $testUpdate = $exported.updates[0]
            $json = @{
                updates = @(
                    @{
                        id = $testUpdate.id
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress

            $out = $json | dsc resource set -r $resourceType -f - -w 2>&1
            $LASTEXITCODE | Should -Be 0

            $result = $out | ConvertFrom-Json
            $result.afterState.updates[0].id | Should -Be $testUpdate.id
            $result.afterState.updates[0].isInstalled | Should -Be $true
            $result.afterState.updates[0]._metadata | Should -BeNullOrEmpty
        }
    }
}
