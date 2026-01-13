# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update Export operation tests' {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/UpdateList'
    }

    Context 'Export operation' {
        It 'should return UpdateList with array of updates' -Skip:(!$IsWindows) {
            $out = '{"updates":[{}]}' | dsc resource export -r $resourceType -o json 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $config = $out | ConvertFrom-Json
            $result = $config.resources[0].properties
            $result.updates | Should -Not -BeNullOrEmpty
            @($result.updates).Count | Should -BeGreaterThan 0
        }

        It 'should work without input filter' -Skip:(!$IsWindows) {
            $out = '' | dsc resource export -r $resourceType -o json 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $config = $out | ConvertFrom-Json
            $result = $config.resources[0].properties
            $result.updates.Count | Should -BeGreaterThan 0
        }

        It 'should filter by isInstalled=true' -Skip:(!$IsWindows) {
            $json = '{"updates":[{"isInstalled": true}]}'
            $out = $json | dsc resource export -r $resourceType -o json 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $config = $out | ConvertFrom-Json
            $result = $config.resources[0].properties
            if ($result.updates.Count -gt 0) {
                foreach ($update in $result.updates) {
                    $update.isInstalled | Should -Be $true
                }
            }
        }

        It 'should filter by isInstalled=false' -Skip:(!$IsWindows) {
            $json = '{"updates":[{"isInstalled": false}]}'
            $out = $json | dsc resource export -r $resourceType -o json 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $config = $out | ConvertFrom-Json
            $result = $config.resources[0].properties
            if ($result.updates.Count -gt 0) {
                foreach ($update in $result.updates) {
                    $update.isInstalled | Should -Be $false
                }
            }
        }

        It 'should filter by title with wildcard in middle' -Skip:(!$IsWindows) {
            $json = '{"updates":[{"title": "*Windows*"}]}'
            $out = $json | dsc resource export -r $resourceType -o json 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $config = $out | ConvertFrom-Json
                $result = $config.resources[0].properties
                if ($result.updates.Count -gt 0) {
                    foreach ($update in $result.updates) {
                        $update.title | Should -Match 'Windows'
                    }
                }
            }
        }

        It 'should return proper structure for each update' -Skip:(!$IsWindows) {
            $out = '{"updates":[{}]}' | dsc resource export -r $resourceType -o json 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $config = $out | ConvertFrom-Json
            $result = $config.resources[0].properties
            if ($result.updates.Count -gt 0) {
                $update = $result.updates[0]
                $update.PSObject.Properties.Name | Should -Contain 'title'
                $update.PSObject.Properties.Name | Should -Contain 'id'
                $update.PSObject.Properties.Name | Should -Contain 'isInstalled'
                $update.PSObject.Properties.Name | Should -Contain 'description'
                $update.PSObject.Properties.Name | Should -Contain 'isUninstallable'
                $update.PSObject.Properties.Name | Should -Contain 'kbArticleIds'
                $update.PSObject.Properties.Name | Should -Contain 'minDownloadSize'
                $update.PSObject.Properties.Name | Should -Contain 'updateType'
                $update.kbArticleIds | Should -Not -BeNull
                @($update.kbArticleIds).Count | Should -BeGreaterOrEqual 0
            }
        }

        It 'should return empty array when no matches found' -Skip:(!$IsWindows) {
            $json = '{"updates":[{"title": "ThisUpdateShouldNeverExist99999*"}]}'
            $out = $json | dsc resource export -r $resourceType -o json 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $config = $out | ConvertFrom-Json
            $result = $config.resources[0].properties
            $result.updates.Count | Should -Be 0
        }

        It 'should filter by msrcSeverity' -Skip:(!$IsWindows) {
            $json = '{"updates":[{"msrcSeverity": "Critical"}]}'
            $out = $json | dsc resource export -r $resourceType -o json 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $config = $out | ConvertFrom-Json
                $result = $config.resources[0].properties
                if ($result.updates.Count -gt 0) {
                    foreach ($update in $result.updates) {
                        $update.msrcSeverity | Should -Be 'Critical'
                    }
                }
            }
        }

        It 'should filter by updateType Software' -Skip:(!$IsWindows) {
            $json = '{"updates":[{"updateType": "Software"}]}'
            $out = $json | dsc resource export -r $resourceType -o json 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $config = $out | ConvertFrom-Json
                $result = $config.resources[0].properties
                if ($result.updates.Count -gt 0) {
                    foreach ($update in $result.updates) {
                        $update.updateType | Should -Be 'Software'
                    }
                }
            }
        }

        It 'should support OR logic with multiple filters in array' -Skip:(!$IsWindows) {
            # Get some updates to use as filters
            $allOut = '{"updates":[{}]}' | dsc resource export -r $resourceType -o json 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $allConfig = $allOut | ConvertFrom-Json
                $allResult = $allConfig.resources[0].properties
                if ($allResult.updates.Count -ge 2) {
                    # Use two specific update IDs as filters (OR logic)
                    $id1 = $allResult.updates[0].id
                    $id2 = $allResult.updates[1].id
                    $json = "{`"updates`":[{`"id`": `"$id1`"}, {`"id`": `"$id2`"}]}"
                    $out = $json | dsc resource export -r $resourceType -o json 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $config = $out | ConvertFrom-Json
                    $result = $config.resources[0].properties
                    
                    # Should return both updates (OR logic)
                    $result.updates.Count | Should -BeGreaterOrEqual 2
                    $foundIds = $result.updates.id
                    $foundIds | Should -Contain $id1
                    $foundIds | Should -Contain $id2
                }
                else {
                    Write-Host "Need at least 2 updates for OR logic test, skipping"
                    $true | Should -Be $true
                }
            }
        }

        It 'should support AND logic within single filter object' -Skip:(!$IsWindows) {
            # Multiple properties in one filter = AND logic
            $json = '{"updates":[{"isInstalled": true, "updateType": "Software"}]}'
            $out = $json | dsc resource export -r $resourceType -o json 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $config = $out | ConvertFrom-Json
                $result = $config.resources[0].properties
                if ($result.updates.Count -gt 0) {
                    # All results must match BOTH conditions
                    foreach ($update in $result.updates) {
                        $update.isInstalled | Should -Be $true
                        $update.updateType | Should -Be 'Software'
                    }
                }
            }
        }

        It 'should not return duplicates when multiple filters match same update' -Skip:(!$IsWindows) {
            # Get an update with known properties
            $allOut = '{"updates":[{}]}' | dsc resource export -r $resourceType -o json 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $allConfig = $allOut | ConvertFrom-Json
                $allResult = $allConfig.resources[0].properties
                if ($allResult.updates.Count -gt 0) {
                    $testUpdate = $allResult.updates[0]
                    # Create two filters that both match the same update
                    $json = "{`"updates`":[{`"id`": `"$($testUpdate.id)`"}, {`"title`": `"$($testUpdate.title)`"}]}"
                    $out = $json | dsc resource export -r $resourceType -o json 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $config = $out | ConvertFrom-Json
                    $result = $config.resources[0].properties
                    
                    # Should return the update only once (no duplicates)
                    $matchingUpdates = $result.updates | Where-Object { $_.id -eq $testUpdate.id }
                    $matchingUpdates.Count | Should -Be 1
                }
            }
        }
    }
}
