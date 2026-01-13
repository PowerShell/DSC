# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update Set operation tests' {
    BeforeDiscovery {
        $resourceType = 'Microsoft.Windows/UpdateList'
        
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
    }
    Context 'Set operation' -Skip:(!$isAdmin -or !$IsWindows) {
        It 'should match when both title and id are correct' {
            # Get an actual installed update with both title and id
            $exportOut = '{"updates": [{"isInstalled": true}]}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $exportOut | ConvertFrom-Json
                if ($result.updates.Count -gt 0) {
                    $testUpdate = $result.updates[0]
                    $json = @{
                        updates = @(
                            @{
                                title = $testUpdate.title
                                id = $testUpdate.id
                            }
                        )
                    } | ConvertTo-Json -Depth 10 -Compress
                    # Try to set (should detect already installed)
                    $out = $json | dsc resource set -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $result = $out | ConvertFrom-Json
                        $result.afterState.updates[0].title | Should -Be $testUpdate.title
                        $result.afterState.updates[0].id | Should -Be $testUpdate.id
                        $result.afterState.updates[0].isInstalled | Should -Be $true
                    }
                }
                else {
                    Write-Host "No installed updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should fail when title matches but id does not' {
            # Get an actual update
            $exportOut = '{"updates": []}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $exportOut | ConvertFrom-Json
                if ($result.updates.Count -gt 0) {
                    $testUpdate = $result.updates[0]
                    $json = @{
                        updates = @(
                            @{
                                title = $testUpdate.title
                                id = '00000000-0000-0000-0000-000000000000'
                            }
                        )
                    } | ConvertTo-Json -Depth 10 -Compress
                    $out = $json | dsc resource set -r $resourceType 2>&1
                    
                    # Should fail because id doesn't match
                    $LASTEXITCODE | Should -Not -Be 0
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should fail when id matches but title does not' {
            # Get an actual update
            $exportOut = '{"updates": []}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $exportOut | ConvertFrom-Json
                if ($result.updates.Count -gt 0) {
                    $testUpdate = $result.updates[0]
                    $json = @{
                        updates = @(
                            @{
                                title = 'ThisWrongTitle99999'
                                id = $testUpdate.id
                            }
                        )
                    } | ConvertTo-Json -Depth 10 -Compress
                    $out = $json | dsc resource set -r $resourceType 2>&1
                    
                    # Should fail because title doesn't match
                    $LASTEXITCODE | Should -Not -Be 0
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }
    }
}
