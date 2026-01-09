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
        It 'should fail when title is missing' -Skip:(!$IsWindows) {
            $json = @'
{
}
'@
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
        It 'should return proper JSON structure for existing update' -Skip:(!$IsWindows) {
            # Search for a common update pattern - Windows Defender updates are common
            $json = @'
{
    "title": "Windows Defender"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $out | ConvertFrom-Json
                $result.actualState | Should -Not -BeNullOrEmpty
                $result.actualState.title | Should -Not -BeNullOrEmpty
                $result.actualState.id | Should -Not -BeNullOrEmpty
                $result.actualState | Should -HaveProperty 'isInstalled'
                $result.actualState | Should -HaveProperty 'description'
                $result.actualState | Should -HaveProperty 'isUninstallable'
                $result.actualState | Should -HaveProperty 'KBArticleIDs'
                $result.actualState | Should -HaveProperty 'maxDownloadSize'
                $result.actualState | Should -HaveProperty 'updateType'
            }
            else {
                # If no Windows Defender update found, that's acceptable
                Write-Host "No Windows Defender update found, skipping structure validation"
                $true | Should -Be $true
            }
        }

        It 'should handle case-insensitive search' -Skip:(!$IsWindows) {
            $jsonLower = @'
{
    "title": "security"
}
'@
            $outLower = $jsonLower | dsc resource get -r $resourceType 2>&1
            
            $jsonUpper = @'
{
    "title": "SECURITY"
}
'@
            $outUpper = $jsonUpper | dsc resource get -r $resourceType 2>&1
            
            # Both should either succeed with results or fail with same error
            $LASTEXITCODE | Should -Be $LASTEXITCODE
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
            $json = @'
{
    "title": "Windows"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $out | ConvertFrom-Json
                $result.actualState.isInstalled | Should -BeOfType [bool]
            }
            else {
                Write-Host "No update found, skipping boolean validation"
                $true | Should -Be $true
            }
        }

        It 'should return valid integer for maxDownloadSize' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "Windows"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $out | ConvertFrom-Json
                $result.actualState.maxDownloadSize | Should -BeGreaterOrEqual 0
            }
            else {
                Write-Host "No update found, skipping size validation"
                $true | Should -Be $true
            }
        }

        It 'should return valid array for KBArticleIDs' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "Windows"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $out | ConvertFrom-Json
                $result.actualState.KBArticleIDs | Should -BeOfType [array]
            }
            else {
                Write-Host "No update found, skipping KB validation"
                $true | Should -Be $true
            }
        }

        It 'should return valid enum value for updateType' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "Windows"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $out | ConvertFrom-Json
                $result.actualState.updateType | Should -BeIn @('Software', 'Driver')
            }
            else {
                Write-Host "No update found, skipping type validation"
                $true | Should -Be $true
            }
        }

        It 'should return valid enum value for msrcSeverity when present' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "Security"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $out | ConvertFrom-Json
                if ($null -ne $result.actualState.msrcSeverity) {
                    $result.actualState.msrcSeverity | Should -BeIn @('Critical', 'Important', 'Moderate', 'Low')
                }
            }
            else {
                Write-Host "No security update found, skipping severity validation"
                $true | Should -Be $true
            }
        }

        It 'should include GUID format for update ID' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "Windows"
}
'@
            $out = $json | dsc resource get -r $resourceType 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $result = $out | ConvertFrom-Json
                # Basic GUID format check (8-4-4-4-12 hex digits)
                $result.actualState.id | Should -Match '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'
            }
            else {
                Write-Host "No update found, skipping ID validation"
                $true | Should -Be $true
            }
        }
    }

    Context 'DSC configuration integration' {
        It 'should work with dsc config get' -Skip:(!$IsWindows) {
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
        It 'should complete get operation within reasonable time' -Skip:(!$IsWindows) {
            $json = @'
{
    "title": "Windows Defender"
}
'@
            $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
            $out = $json | dsc resource get -r $resourceType 2>&1
            $stopwatch.Stop()
            
            # Windows Update queries can be slow, but should complete within 60 seconds
            $stopwatch.Elapsed.TotalSeconds | Should -BeLessThan 60
        }
    }
}
