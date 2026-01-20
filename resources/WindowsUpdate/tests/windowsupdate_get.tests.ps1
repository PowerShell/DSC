# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update Get operation tests' {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/UpdateList'
        $result = dsc resource export -r $resourceType | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $exportOut = $result.resources[0].properties
        $exportOut.updates.Count | Should -BeGreaterThan 0
    }

    Context 'Get operation' {
        It 'should return proper JSON structure for existing update with exact title' -Skip:(!$IsWindows) {
            $exactTitle = $exportOut.updates[0].title
            $json = @{
                updates = @(
                    @{
                        title = $exactTitle
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $getResult = $out | ConvertFrom-Json
            $getResult.actualState | Should -Not -BeNullOrEmpty
            $getResult.actualState.updates[0].title | Should -BeExactly $exactTitle
            $getResult.actualState.updates[0].id | Should -Not -BeNullOrEmpty
            $getResult.actualState.updates[0].isInstalled | Should -BeIn ($true, $false)
            $getResult.actualState.updates[0].description | Should -Not -BeNullOrEmpty
            $getResult.actualState.updates[0].isUninstallable | Should -BeIn ($true, $false)
            $getResult.actualState.updates[0].recommendedHardDiskSpace | Should -BeGreaterOrEqual 0
            $getResult.actualState.updates[0].updateType | Should -BeIn @('Software', 'Driver')
        }

        It 'should handle case-insensitive exact title match' -Skip:(!$IsWindows) {
            $exactTitle = $exportOut.updates[0].title
            
            # Test with lowercase version
            $jsonLower = @{
                updates = @(
                    @{
                        title = $exactTitle.ToLower()
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $outLower = $jsonLower | dsc resource get -r $resourceType 2>&1
            
            # Test with uppercase version
            $jsonUpper = @{
                updates = @(
                    @{
                        title = $exactTitle.ToUpper()
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $outUpper = $jsonUpper | dsc resource get -r $resourceType 2>&1
            
            # Both should succeed
            if ($outLower -and $outUpper) {
                $resultLower = $outLower | ConvertFrom-Json
                $resultUpper = $outUpper | ConvertFrom-Json
                $resultLower.actualState.updates[0].id | Should -Be $resultUpper.actualState.updates[0].id
            }
        }

        It 'should fail when partial title is provided' -Skip:(!$IsWindows) {
            # Get operation now requires exact match, so partial should fail
            $json = @{
                updates = @(
                    @{
                        title = 'Windows'
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource get -r $resourceType 2>&1
            # This will likely fail unless there's an update with exact title "Windows"
            # which is unlikely
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should fail when update is not found' -Skip:(!$IsWindows) {
            # Use a very unlikely update title
            $json = @{
                updates = @(
                    @{
                        title = 'ThisUpdateShouldNeverExist12345XYZ'
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should match when both title and id are correct' -Skip:(!$IsWindows) {
            $testUpdate = $exportOut.updates[0]
            $json = @{
                updates = @(
                    @{
                        title = $testUpdate.title
                        id = $testUpdate.id
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result.actualState.updates[0].title | Should -Be $testUpdate.title
            $result.actualState.updates[0].id | Should -Be $testUpdate.id
        }

        It 'should fail when title matches but id does not' -Skip:(!$IsWindows) {
            $testUpdate = $exportOut.updates[0]
            $json = @{
                updates = @(
                    @{
                        title = $testUpdate.title
                        id = '00000000-0000-0000-0000-000000000000'
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource get -r $resourceType 2>&1
            
            # Should fail because id doesn't match
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should fail when id matches but title does not' -Skip:(!$IsWindows) {
            $testUpdate = $exportOut.updates[0]
            $json = @{
                updates = @(
                    @{
                        title = 'ThisWrongTitle99999'
                        id = $testUpdate.id
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $null = $json | dsc resource get -r $resourceType 2>&1
            
            # Should fail because title doesn't match
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should return valid boolean for isInstalled' -Skip:(!$IsWindows) {
            $json = @{
                updates = @(
                    @{
                        title = $exportOut.updates[0].title
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Be 0            
            $result = $out | ConvertFrom-Json
            $result.actualState.updates[0].isInstalled | Should -BeOfType [bool]
        }

        It 'should return valid integer for recommendedHardDiskSpace' -Skip:(!$IsWindows) {
            $json = @{
                updates = @(
                    @{
                        title = $exportOut.updates[0].title
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Be 0            
            $result = $out | ConvertFrom-Json
            $result.actualState.updates[0].recommendedHardDiskSpace | Should -BeGreaterOrEqual 0
        }

        It 'should return valid array for KBArticleIDs' -Skip:(!$IsWindows) {
            $json = @{
                updates = @(
                    @{
                        title = $exportOut.updates[0].title
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Be 0            
            $result = $out | ConvertFrom-Json
            $result.actualState.updates[0].kbArticleIds.GetType().BaseType.Name | Should -Be 'Array'
        }

        It 'should return valid enum value for updateType' -Skip:(!$IsWindows) {
            $json = @{
                updates = @(
                    @{
                        title = $exportOut.updates[0].title
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Be 0            
            $result = $out | ConvertFrom-Json
            $result.actualState.updates[0].updateType | Should -BeIn @('Software', 'Driver')
        }

        It 'should return valid enum value for msrcSeverity when present' -Skip:(!$IsWindows) {
            $updateWithSeverity = $exportOut.updates | Where-Object { $null -ne $_.msrcSeverity } | Select-Object -First 1
                
            if ($updateWithSeverity) {
                $json = @{
                    updates = @(
                        @{
                            title = $updateWithSeverity.title
                        }
                    )
                } | ConvertTo-Json -Depth 10 -Compress
                $out = $json | dsc resource get -r $resourceType 2>&1
                $LASTEXITCODE | Should -Be 0
                $result = $out | ConvertFrom-Json
                $result.actualState.updates[0].msrcSeverity | Should -BeExactly $updateWithSeverity.msrcSeverity
            }
        }

        It 'should include GUID format for update ID' -Skip:(!$IsWindows) {
            $json = @{
                updates = @(
                    @{
                        title = $exportOut.updates[0].title
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
                    
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            # Basic GUID format check (8-4-4-4-12 hex digits)
            $result.actualState.updates[0].id | Should -Match '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
        }

        It 'should support lookup by id' -Skip:(!$IsWindows) {
            $updateId = $exportOut.updates[0].id
            $json = @{
                updates = @(
                    @{
                        id = $updateId
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
                    
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            $result.actualState.updates[0].id | Should -Be $updateId
        }

        It 'should process multiple input objects and return all matches' -Skip:(!$IsWindows) {
            # Get at least 2 updates to test with
            if ($exportOut.updates.Count -ge 2) {
                $update1 = $exportOut.updates[0]
                $update2 = $exportOut.updates[1]
                
                $json = @{
                    updates = @(
                        @{
                            title = $update1.title
                        },
                        @{
                            title = $update2.title
                        }
                    )
                } | ConvertTo-Json -Depth 10 -Compress
                $out = $json | dsc resource get -r $resourceType 2>&1
                
                $LASTEXITCODE | Should -Be 0
                $getResult = $out | ConvertFrom-Json
                $getResult.actualState.updates.Count | Should -Be 2
                $getResult.actualState.updates[0].title | Should -BeIn @($update1.title, $update2.title)
                $getResult.actualState.updates[1].title | Should -BeIn @($update1.title, $update2.title)
            } else {
                Set-ItResult -Skipped -Because "Need at least 2 updates for this test"
            }
        }

        It 'should fail if any input object does not have a match' -Skip:(!$IsWindows) {
            $update1 = $exportOut.updates[0]
            
            $json = @{
                updates = @(
                    @{
                        title = $update1.title
                    },
                    @{
                        title = 'ThisUpdateShouldNeverExist12345XYZ'
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $stderr = $json | dsc resource get -r $resourceType 2>&1
            
            # Should fail because second input has no match
            $LASTEXITCODE | Should -Not -Be 0
            
            # Check for error message in stderr
            $errorText = $stderr | Out-String
            $errorText | Should -Match 'No matching update found'
        }

        It 'should support filtering by KB article IDs' -Skip:(!$IsWindows) {
            # Find an update with KB article IDs
            $updateWithKB = $exportOut.updates | Where-Object { $_.kbArticleIds.Count -gt 0 } | Select-Object -First 1
            
            if ($updateWithKB) {
                $json = @{
                    updates = @(
                        @{
                            kbArticleIds = @($updateWithKB.kbArticleIds[0])
                        }
                    )
                } | ConvertTo-Json -Depth 10 -Compress
                $out = $json | dsc resource get -r $resourceType 2>&1
                
                $LASTEXITCODE | Should -Be 0
                $getResult = $out | ConvertFrom-Json
                $getResult.actualState.updates[0].kbArticleIds | Should -Contain $updateWithKB.kbArticleIds[0]
            } else {
                Set-ItResult -Skipped -Because "No updates with KB article IDs found"
            }
        }

        It 'should support filtering by update type' -Skip:(!$IsWindows) {
            $softwareUpdate = $exportOut.updates | Where-Object { $_.updateType -eq 'Software' } | Select-Object -First 1
            
            if ($softwareUpdate) {
                $json = @{
                    updates = @(
                        @{
                            id = $softwareUpdate.id
                            updateType = 'Software'
                        }
                    )
                } | ConvertTo-Json -Depth 10 -Compress
                $out = $json | dsc resource get -r $resourceType 2>&1
                
                $LASTEXITCODE | Should -Be 0
                $getResult = $out | ConvertFrom-Json
                $getResult.actualState.updates[0].updateType | Should -Be 'Software'
            } else {
                Set-ItResult -Skipped -Because "No software updates found"
            }
        }

        It 'should support filtering by MSRC severity with AND logic' -Skip:(!$IsWindows) {
            $updateWithSeverity = $exportOut.updates | Where-Object { $null -ne $_.msrcSeverity } | Select-Object -First 1
            
            if ($updateWithSeverity) {
                $json = @{
                    updates = @(
                        @{
                            id = $updateWithSeverity.id
                            msrcSeverity = $updateWithSeverity.msrcSeverity
                        }
                    )
                } | ConvertTo-Json -Depth 10 -Compress
                $out = $json | dsc resource get -r $resourceType 2>&1
                
                $LASTEXITCODE | Should -Be 0
                $getResult = $out | ConvertFrom-Json
                $getResult.actualState.updates[0].msrcSeverity | Should -Be $updateWithSeverity.msrcSeverity
            } else {
                Set-ItResult -Skipped -Because "No updates with MSRC severity found"
            }
        }

        It 'should return installationBehavior property when present' -Skip:(!$IsWindows) {
            $json = @{
                updates = @(
                    @{
                        title = $exportOut.updates[0].title
                    }
                )
            } | ConvertTo-Json -Depth 10 -Compress
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $result = $out | ConvertFrom-Json
            # installationBehavior should be one of the valid enum values if present
            if ($null -ne $result.actualState.updates[0].installationBehavior) {
                $result.actualState.updates[0].installationBehavior | Should -BeIn @('NeverReboots', 'AlwaysRequiresReboot', 'CanRequestReboot')
            }
        }

        It 'should return valid enum value for installationBehavior' -Skip:(!$IsWindows) {
            # Find an update that has installationBehavior set
            $updateWithBehavior = $exportOut.updates | Where-Object { $null -ne $_.installationBehavior } | Select-Object -First 1
            
            if ($updateWithBehavior) {
                $json = @{
                    updates = @(
                        @{
                            id = $updateWithBehavior.id
                        }
                    )
                } | ConvertTo-Json -Depth 10 -Compress
                $out = $json | dsc resource get -r $resourceType 2>&1
                
                $LASTEXITCODE | Should -Be 0
                $getResult = $out | ConvertFrom-Json
                $getResult.actualState.updates[0].installationBehavior | Should -BeIn @('NeverReboots', 'AlwaysRequiresReboot', 'CanRequestReboot')
            } else {
                Set-ItResult -Skipped -Because "No updates with installationBehavior found"
            }
        }

        It 'should fail when title matches multiple updates' -Skip:(!$IsWindows) {
            # Find a title pattern that might match multiple updates
            # Using isInstalled filter with a common partial title like 'Windows' could match multiple
            # This test verifies the new multiple-match detection behavior
            
            # First, check if there are multiple updates with similar titles
            $windowsUpdates = $exportOut.updates | Where-Object { $_.title -like '*Windows*' }
            
            if ($windowsUpdates.Count -ge 2) {
                # Find a common substring that appears in multiple update titles
                # Try to use a very generic criteria that would match multiple
                $json = @{
                    updates = @(
                        @{
                            isInstalled = $true
                        }
                    )
                } | ConvertTo-Json -Depth 10 -Compress
                $stderr = $json | dsc resource get -r $resourceType 2>&1
                
                # If multiple updates match isInstalled=true, it should error
                $installedCount = ($exportOut.updates | Where-Object { $_.isInstalled -eq $true }).Count
                if ($installedCount -gt 1) {
                    $LASTEXITCODE | Should -Not -Be 0
                    $errorText = $stderr | Out-String
                    $errorText | Should -Match 'matched.*updates|multiple'
                } else {
                    # Only one installed update, so it should succeed
                    $LASTEXITCODE | Should -Be 0
                }
            } else {
                Set-ItResult -Skipped -Because "Need multiple updates to test multiple match detection"
            }
        }

        It 'should provide helpful error message when multiple updates match title criteria' -Skip:(!$IsWindows) {
            # Find a case where using title-only might match multiple updates
            # Group updates by similar starting titles
            $titleGroups = $exportOut.updates | Group-Object { ($_.title -split ' ')[0..2] -join ' ' } | Where-Object { $_.Count -gt 1 }
            
            if ($titleGroups.Count -gt 0) {
                # Use the first duplicate-ish title group
                $firstGroup = $titleGroups[0].Group
                if ($firstGroup.Count -ge 2) {
                    # There might be multiple updates with same starting title
                    # The error message should mention using more specific identifiers
                    $json = @{
                        updates = @(
                            @{
                                isInstalled = $firstGroup[0].isInstalled
                                updateType = $firstGroup[0].updateType
                            }
                        )
                    } | ConvertTo-Json -Depth 10 -Compress
                    $stderr = $json | dsc resource get -r $resourceType 2>&1
                    
                    # This may or may not fail depending on uniqueness
                    if ($LASTEXITCODE -ne 0) {
                        $errorText = $stderr | Out-String
                        # Should contain helpful guidance
                        $errorText | Should -Match 'specific|identifier|criteria'
                    }
                }
            } else {
                Set-ItResult -Skipped -Because "No duplicate title patterns found for testing"
            }
        }
    }
}
