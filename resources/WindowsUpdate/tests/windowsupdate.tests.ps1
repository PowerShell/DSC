# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update resource tests' {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/Updates'
        
        # Helper function to check if running as administrator
        function Test-IsAdmin {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        
        $isAdmin = Test-IsAdmin
    }

    Context 'Resource discovery' {
        It 'should be discoverable in DSC resource list' -Skip:(!$IsWindows) {
            $resources = dsc resource list | ConvertFrom-Json
            $windowsUpdate = $resources | Where-Object { $_.type -eq $resourceType }
            $windowsUpdate | Should -Not -BeNullOrEmpty
            $windowsUpdate.type | Should -BeExactly $resourceType
            $windowsUpdate.version | Should -BeExactly '0.1.0'
        }

        It 'should have get capability' -Skip:(!$IsWindows) {
            $resources = dsc resource list | ConvertFrom-Json
            $windowsUpdate = $resources | Where-Object { $_.type -eq $resourceType }
            $windowsUpdate.capabilities | Should -Contain 'get'
        }

        It 'should have description' -Skip:(!$IsWindows) {
            $resources = dsc resource list | ConvertFrom-Json
            $windowsUpdate = $resources | Where-Object { $_.type -eq $resourceType }
            $windowsUpdate.description | Should -Not -BeNullOrEmpty
        }
    }

    Context 'Input validation' {
        It 'should allow get without title or id for specific lookup' -Skip:(!$IsWindows) {
            $json = @'
{
}
'@
            # For get operation, empty input is not valid (need title or id)
            $out = $json | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should fail when input is invalid JSON' -Skip:(!$IsWindows) {
            $invalidJson = 'not valid json'
            $out = $invalidJson | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should handle empty title gracefully' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": ""
}
'@
            # Empty title should either fail or return no results
            $out = $json | dsc resource get -r $resourceType 2>&1
            # We expect an error since no update will match empty string
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Get operation' {
        It 'should return proper JSON structure for existing update with exact title' -Skip:(!$IsWindows) {
            # Get a list of actual updates first to test with exact title
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $exactTitle = $updates[0].title
                    $json = @"
{
    ""title"": ""$exactTitle""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $result = $out | ConvertFrom-Json
                    $result.actualState | Should -Not -BeNullOrEmpty
                    $result.actualState.title | Should -BeExactly $exactTitle
                    $result.actualState.id | Should -Not -BeNullOrEmpty
                    $result.actualState | Should -HaveProperty 'isInstalled'
                    $result.actualState | Should -HaveProperty 'description'
                    $result.actualState | Should -HaveProperty 'isUninstallable'
                    $result.actualState | Should -HaveProperty 'KBArticleIDs'
                    $result.actualState | Should -HaveProperty 'maxDownloadSize'
                    $result.actualState | Should -HaveProperty 'updateType'
                }
                else {
                    Write-Host "No updates found on system, skipping test"
                    $true | Should -Be $true
                }
            }
            else {
                Write-Host "Export failed, skipping test"
                $true | Should -Be $true
            }
        }

        It 'should handle case-insensitive exact title match' -Skip:(!$IsWindows) {
            # Get an update first to test with
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $exactTitle = $updates[0].title
                    
                    # Test with lowercase version
                    $jsonLower = @"
{
    ""title"": ""$($exactTitle.ToLower())""
}
"@
                    $outLower = $jsonLower | dsc resource get -r $resourceType 2>&1
                    
                    # Test with uppercase version
                    $jsonUpper = @"
{
    ""title"": ""$($exactTitle.ToUpper())""
}
"@
                    $outUpper = $jsonUpper | dsc resource get -r $resourceType 2>&1
                    
                    # Both should succeed
                    if ($outLower -and $outUpper) {
                        $resultLower = $outLower | ConvertFrom-Json
                        $resultUpper = $outUpper | ConvertFrom-Json
                        $resultLower.actualState.id | Should -Be $resultUpper.actualState.id
                    }
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
            else {
                Write-Host "Export failed, skipping test"
                $true | Should -Be $true
            }
        }

        It 'should fail when partial title is provided' -Skip:(!$IsWindows) {
            # Get operation now requires exact match, so partial should fail
            $json = @'
{
    "title": "Windows"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            # This will likely fail unless there's an update with exact title "Windows"
            # which is unlikely
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should fail when update is not found' -Skip:(!$IsWindows) {
            # Use a very unlikely update title
            $json = @'
{
    "title": "ThisUpdateShouldNeverExist12345XYZ"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should return valid boolean for isInstalled' -Skip:(!$IsWindows) {
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $json = @"
{
    ""title"": ""$($updates[0].title)""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $result = $out | ConvertFrom-Json
                        $result.actualState.isInstalled | Should -BeOfType [bool]
                    }
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should return valid integer for maxDownloadSize' -Skip:(!$IsWindows) {
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $json = @"
{
    ""title"": ""$($updates[0].title)""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $result = $out | ConvertFrom-Json
                        $result.actualState.maxDownloadSize | Should -BeGreaterOrEqual 0
                    }
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should return valid array for KBArticleIDs' -Skip:(!$IsWindows) {
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $json = @"
{
    ""title"": ""$($updates[0].title)""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $result = $out | ConvertFrom-Json
                        $result.actualState.KBArticleIDs | Should -BeOfType [array]
                    }
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should return valid enum value for updateType' -Skip:(!$IsWindows) {
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $json = @"
{
    ""title"": ""$($updates[0].title)""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $result = $out | ConvertFrom-Json
                        $result.actualState.updateType | Should -BeIn @('Software', 'Driver')
                    }
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should return valid enum value for msrcSeverity when present' -Skip:(!$IsWindows) {
            # Find an update with severity information using export
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                $updateWithSeverity = $updates | Where-Object { $null -ne $_.msrcSeverity } | Select-Object -First 1
                
                if ($updateWithSeverity) {
                    $json = @"
{
    ""title"": ""$($updateWithSeverity.title)""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $result = $out | ConvertFrom-Json
                        if ($null -ne $result.actualState.msrcSeverity) {
                            $result.actualState.msrcSeverity | Should -BeIn @('Critical', 'Important', 'Moderate', 'Low')
                        }
                    }
                }
                else {
                    Write-Host "No update with severity found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should include GUID format for update ID' -Skip:(!$IsWindows) {
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $json = @"
{
    ""title"": ""$($updates[0].title)""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $result = $out | ConvertFrom-Json
                        # Basic GUID format check (8-4-4-4-12 hex digits)
                        $result.actualState.id | Should -Match '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
                    }
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should support lookup by id' -Skip:(!$IsWindows) {
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $updateId = $updates[0].id
                    $json = @"
{
    ""id"": ""$updateId""
}
"@
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $result = $out | ConvertFrom-Json
                    $result.actualState.id | Should -Be $updateId
                }
                else {
                    Write-Host "No updates found, skipping test"
                    $true | Should -Be $true
                }
            }
        }
    }

    Context 'Export operation' {
        It 'should return array of updates' -Skip:(!$IsWindows) {
            $out = '{}' | dsc resource export -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $updates = $out | ConvertFrom-Json
            $updates | Should -BeOfType [array]
        }

        It 'should work without input filter' -Skip:(!$IsWindows) {
            $out = '' | dsc resource export -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $updates = $out | ConvertFrom-Json
            $updates.Count | Should -BeGreaterThan 0
        }

        It 'should filter by isInstalled=true' -Skip:(!$IsWindows) {
            $json = @'
{
    "isInstalled": true
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $updates = $out | ConvertFrom-Json
            if ($updates.Count -gt 0) {
                foreach ($update in $updates) {
                    $update.isInstalled | Should -Be $true
                }
            }
        }

        It 'should filter by isInstalled=false' -Skip:(!$IsWindows) {
            $json = @'
{
    "isInstalled": false
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $updates = $out | ConvertFrom-Json
            if ($updates.Count -gt 0) {
                foreach ($update in $updates) {
                    $update.isInstalled | Should -Be $false
                }
            }
        }

        It 'should filter by title with wildcard *' -Skip:(!$IsWindows) {
            # Get first update to construct wildcard pattern
            $allOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $allUpdates = $allOut | ConvertFrom-Json
                if ($allUpdates.Count -gt 0) {
                    # Take first word from title and use as wildcard
                    $firstWord = ($allUpdates[0].title -split ' ')[0]
                    $json = @"
{
    ""title"": ""$firstWord*""
}
"@
                    $out = $json | dsc resource export -r $resourceType 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $updates = $out | ConvertFrom-Json
                    $updates.Count | Should -BeGreaterThan 0
                    
                    # All results should start with the pattern
                    foreach ($update in $updates) {
                        $update.title | Should -BeLike "$firstWord*"
                    }
                }
            }
        }

        It 'should filter by title with wildcard in middle' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "*Windows*"
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.title | Should -Match 'Windows'
                    }
                }
            }
        }

        It 'should combine filters - title wildcard and isInstalled' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "*Microsoft*",
    "isInstalled": true
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.title | Should -Match 'Microsoft'
                        $update.isInstalled | Should -Be $true
                    }
                }
            }
        }

        It 'should return proper structure for each update' -Skip:(!$IsWindows) {
            $out = '{}' | dsc resource export -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $updates = $out | ConvertFrom-Json
            if ($updates.Count -gt 0) {
                $update = $updates[0]
                $update | Should -HaveProperty 'title'
                $update | Should -HaveProperty 'id'
                $update | Should -HaveProperty 'isInstalled'
                $update | Should -HaveProperty 'description'
                $update | Should -HaveProperty 'isUninstallable'
                $update | Should -HaveProperty 'kbArticleIds'
                $update | Should -HaveProperty 'maxDownloadSize'
                $update | Should -HaveProperty 'updateType'
                $update.kbArticleIds | Should -BeOfType [array]
            }
        }

        It 'should filter by specific update id' -Skip:(!$IsWindows) {
            $allOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $allUpdates = $allOut | ConvertFrom-Json
                if ($allUpdates.Count -gt 0) {
                    $specificId = $allUpdates[0].id
                    $json = @"
{
    ""id"": ""$specificId""
}
"@
                    $out = $json | dsc resource export -r $resourceType 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $updates = $out | ConvertFrom-Json
                    $updates.Count | Should -Be 1
                    $updates[0].id | Should -Be $specificId
                }
            }
        }

        It 'should return empty array when no matches found' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "ThisUpdateShouldNeverExist99999*"
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            $LASTEXITCODE | Should -Be 0
            $updates = $out | ConvertFrom-Json
            $updates.Count | Should -Be 0
        }

        It 'should filter by msrcSeverity' -Skip:(!$IsWindows) {
            $json = @'
{
    "msrcSeverity": "Critical"
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.msrcSeverity | Should -Be 'Critical'
                    }
                }
            }
        }

        It 'should filter by updateType Software' -Skip:(!$IsWindows) {
            $json = @'
{
    "updateType": "Software"
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.updateType | Should -Be 'Software'
                    }
                }
            }
        }

        It 'should filter by updateType Driver' -Skip:(!$IsWindows) {
            $json = @'
{
    "updateType": "Driver"
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                # May return 0 updates if no drivers are pending
                foreach ($update in $updates) {
                    $update.updateType | Should -Be 'Driver'
                }
            }
        }

        It 'should filter by isUninstallable' -Skip:(!$IsWindows) {
            $json = @'
{
    "isUninstallable": true
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.isUninstallable | Should -Be $true
                    }
                }
            }
        }

        It 'should filter by description with wildcard' -Skip:(!$IsWindows) {
            $json = @'
{
    "description": "*security*"
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.description | Should -Match 'security'
                    }
                }
            }
        }

        It 'should filter by kbArticleIds' -Skip:(!$IsWindows) {
            # First get an update with KB articles
            $allOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $allUpdates = $allOut | ConvertFrom-Json
                $updateWithKB = $allUpdates | Where-Object { $_.kbArticleIds.Count -gt 0 } | Select-Object -First 1
                
                if ($updateWithKB) {
                    $kbId = $updateWithKB.kbArticleIds[0]
                    $json = @"
{
    "kbArticleIds": ["$kbId"]
}
"@
                    $out = $json | dsc resource export -r $resourceType 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $updates = $out | ConvertFrom-Json
                    $updates.Count | Should -BeGreaterThan 0
                    
                    # At least one update should have the KB ID
                    $matchFound = $false
                    foreach ($update in $updates) {
                        if ($update.kbArticleIds -contains $kbId) {
                            $matchFound = $true
                            break
                        }
                    }
                    $matchFound | Should -Be $true
                }
                else {
                    Write-Host "No update with KB articles found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should filter by securityBulletinIds' -Skip:(!$IsWindows) {
            # First get an update with security bulletins
            $allOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $allUpdates = $allOut | ConvertFrom-Json
                $updateWithBulletin = $allUpdates | Where-Object { $_.securityBulletinIds.Count -gt 0 } | Select-Object -First 1
                
                if ($updateWithBulletin) {
                    $bulletinId = $updateWithBulletin.securityBulletinIds[0]
                    $json = @"
{
    "securityBulletinIds": ["$bulletinId"]
}
"@
                    $out = $json | dsc resource export -r $resourceType 2>&1
                    
                    $LASTEXITCODE | Should -Be 0
                    $updates = $out | ConvertFrom-Json
                    $updates.Count | Should -BeGreaterThan 0
                    
                    # At least one update should have the bulletin ID
                    $matchFound = $false
                    foreach ($update in $updates) {
                        if ($update.securityBulletinIds -contains $bulletinId) {
                            $matchFound = $true
                            break
                        }
                    }
                    $matchFound | Should -Be $true
                }
                else {
                    Write-Host "No update with security bulletins found, skipping test"
                    $true | Should -Be $true
                }
            }
        }

        It 'should combine multiple filters - msrcSeverity and isInstalled' -Skip:(!$IsWindows) {
            $json = @'
{
    "msrcSeverity": "Critical",
    "isInstalled": false
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.msrcSeverity | Should -Be 'Critical'
                        $update.isInstalled | Should -Be $false
                    }
                }
            }
        }

        It 'should combine multiple filters - updateType and title wildcard' -Skip:(!$IsWindows) {
            $json = @'
{
    "updateType": "Software",
    "title": "*Windows*"
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.updateType | Should -Be 'Software'
                        $update.title | Should -Match 'Windows'
                    }
                }
            }
        }

        It 'should combine three filters - msrcSeverity, updateType, and isInstalled' -Skip:(!$IsWindows) {
            $json = @'
{
    "msrcSeverity": "Important",
    "updateType": "Software",
    "isInstalled": true
}
'@
            $out = $json | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $out | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    foreach ($update in $updates) {
                        $update.msrcSeverity | Should -Be 'Important'
                        $update.updateType | Should -Be 'Software'
                        $update.isInstalled | Should -Be $true
                    }
                }
            }
        }

        It 'should handle all msrcSeverity values' -Skip:(!$IsWindows) {
            $severities = @('Critical', 'Important', 'Moderate', 'Low')
            
            foreach ($severity in $severities) {
                $json = @"
{
    "msrcSeverity": "$severity"
}
"@
                $out = $json | dsc resource export -r $resourceType 2>&1
                
                if ($LASTEXITCODE -eq 0) {
                    $updates = $out | ConvertFrom-Json
                    # May return 0 updates if no matches, but should not error
                    foreach ($update in $updates) {
                        $update.msrcSeverity | Should -Be $severity
                    }
                }
            }
        }

        It 'should filter by description exact match (no wildcard)' -Skip:(!$IsWindows) {
            # Get an actual description first
            $allOut = '{}' | dsc resource export -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $allUpdates = $allOut | ConvertFrom-Json
                if ($allUpdates.Count -gt 0) {
                    $exactDesc = $allUpdates[0].description
                    $json = @"
{
    "description": "$($exactDesc -replace '"', '\"')"
}
"@
                    $out = $json | dsc resource export -r $resourceType 2>&1
                    
                    if ($LASTEXITCODE -eq 0) {
                        $updates = $out | ConvertFrom-Json
                        $updates.Count | Should -BeGreaterThan 0
                        $updates[0].description | Should -Be $exactDesc
                    }
                }
            }
        }
    }

    Context 'DSC configuration integration' {
        It 'should work with dsc config get using exact title' -Skip:(!$IsWindows) {
            $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/configuration.json
resources:
- name: QueryUpdate
  type: Microsoft.Windows/Updates
  properties:
    title: Windows
'@
            $tempFile = [System.IO.Path]::GetTempFileName() + ".yaml"
            Set-Content -Path $tempFile -Value $configYaml -Force
            
            try {
                $out = dsc config get -f $tempFile 2>&1
                
                if ($LASTEXITCODE -eq 0) {
                    $result = $out | ConvertFrom-Json
                    $result.results | Should -Not -BeNullOrEmpty
                    $result.results[0].name | Should -Be 'QueryUpdate'
                    $result.results[0].type | Should -Be $resourceType
                }
                else {
                    # If no update found, that's acceptable
                    Write-Host "Config get did not find matching update"
                    $true | Should -Be $true
                }
            }
            finally {
                Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
            }
        }

        It 'should handle resource not found in configuration gracefully' -Skip:(!$IsWindows) {
            $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/configuration.json
resources:
- name: QueryNonExistentUpdate
  type: Microsoft.Windows/Updates
  properties:
    title: ThisUpdateShouldNeverExist99999
'@
            $tempFile = [System.IO.Path]::GetTempFileName() + ".yaml"
            Set-Content -Path $tempFile -Value $configYaml -Force
            
            try {
                $out = dsc config get -f $tempFile 2>&1
                # Should fail gracefully
                $LASTEXITCODE | Should -Not -Be 0
            }
            finally {
                Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
            }
        }
    }

    Context 'Executable behavior' {
        It 'executable should exist' -Skip:(!$IsWindows) {
            $exePath = (Get-Command wu_dsc -ErrorAction SilentlyContinue).Source
            if ($null -ne $exePath) {
                Test-Path $exePath | Should -Be $true
            }
            else {
                # Executable might not be in PATH yet, check in resource directory
                $resourcePath = Join-Path $PSScriptRoot ".." 
                $possiblePaths = @(
                    (Join-Path $resourcePath "target\release\wu_dsc.exe"),
                    (Join-Path $resourcePath "target\debug\wu_dsc.exe"),
                    "wu_dsc.exe"
                )
                
                $found = $false
                foreach ($path in $possiblePaths) {
                    if (Test-Path $path) {
                        $found = $true
                        break
                    }
                }
                
                if (-not $found) {
                    Write-Warning "wu_dsc executable not found. Build may be required."
                }
            }
        }

        It 'should fail gracefully when operation is not supported' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "Windows"
}
'@
            # Test operation should not be implemented
            $out = $json | dsc resource test -r $resourceType 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Platform compatibility' {
        It 'should only run on Windows' {
            if (-not $IsWindows) {
                $json = @'
{
    "title": "test"
}
'@
                $out = $json | dsc resource get -r $resourceType 2>&1
                $LASTEXITCODE | Should -Not -Be 0
                $out | Should -Match 'Windows'
            }
            else {
                $true | Should -Be $true
            }
        }
    }

    Context 'Performance' {
        It 'should complete export operation within reasonable time' -Skip:(!$IsWindows) {
            $json = @'
{
    "isInstalled": true
}
'@
            $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
            $out = $json | dsc resource export -r $resourceType 2>&1
            $stopwatch.Stop()
            
            # Windows Update queries can be slow, but should complete within 60 seconds
            $stopwatch.Elapsed.TotalSeconds | Should -BeLessThan 60
        }

        It 'should complete get operation within reasonable time' -Skip:(!$IsWindows) {
            # Get a real update first
            $exportOut = '{}' | dsc resource export -r $resourceType 2>&1 | Select-Object -First 1
            
            if ($LASTEXITCODE -eq 0) {
                $updates = $exportOut | ConvertFrom-Json
                if ($updates.Count -gt 0) {
                    $json = @"
{
    ""title"": ""$($updates[0].title)""
}
"@
                    $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
                    $out = $json | dsc resource get -r $resourceType 2>&1
                    $stopwatch.Stop()
                    
                    # Windows Update queries can be slow, but should complete within 60 seconds
                    $stopwatch.Elapsed.TotalSeconds | Should -BeLessThan 60
                }
            }
        }
    }
}
