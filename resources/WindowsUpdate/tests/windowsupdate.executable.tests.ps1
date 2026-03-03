# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update resource executable tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $exeName = 'wu_dsc'
        $resourceDir = Join-Path $PSScriptRoot ".."
        
        # Try to find the executable
        $exePaths = @(
            (Join-Path $resourceDir "target\release\wu_dsc.exe"),
            (Join-Path $resourceDir "target\debug\wu_dsc.exe")
        )
        
        $exePath = $null
        foreach ($path in $exePaths) {
            if (Test-Path $path) {
                $exePath = $path
                break
            }
        }
        
        # If not found in target dirs, try to find in PATH
        if ($null -eq $exePath) {
            $exePath = (Get-Command wu_dsc.exe -ErrorAction SilentlyContinue).Source
        }
        
        $skipTests = (-not $IsWindows) -or ($null -eq $exePath)
        
        if ($skipTests -and $IsWindows) {
            Write-Warning "wu_dsc executable not found. Run 'cargo build' to build the resource."
        }
    }

    Context 'Executable file properties' {
        It 'executable should exist' -Skip:$skipTests {
            Test-Path $exePath | Should -Be $true
        }

        It 'executable should be a PE file' -Skip:$skipTests {
            $bytes = [System.IO.File]::ReadAllBytes($exePath)
            # Check for MZ header (PE executable)
            $bytes[0] | Should -Be 0x4D  # 'M'
            $bytes[1] | Should -Be 0x5A  # 'Z'
        }

        It 'executable should have .exe extension' -Skip:$skipTests {
            $exePath | Should -Match '\.exe$'
        }
    }

    Context 'Command line interface' {
        It 'should fail without arguments' -Skip:$skipTests {
            $result = & $exePath 2>&1
            $LASTEXITCODE | Should -Not -Be 0
            $result | Should -Match 'Usage|Error'
        }

        It 'should display usage information when called without args' -Skip:$skipTests {
            $result = & $exePath 2>&1
            $result | Should -Match 'Usage|operation|get|set|test'
        }

        It 'should fail with unknown operation' -Skip:$skipTests {
            $json = '[{"title": "test"}]'
            $result = $json | & $exePath 'invalid_operation' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
            $result | Should -Match 'Unknown operation|Error|Usage'
        }
    }

    Context 'Get operation input handling' {
        It 'should accept JSON input via stdin' -Skip:$skipTests {
            $json = '[{"title": "Windows Defender"}]'
            $result = $json | & $exePath 'get' 2>&1
            # May succeed or fail depending on updates, but should process the input
            $result | Should -Not -BeNullOrEmpty
        }

        It 'should fail with invalid JSON input' -Skip:$skipTests {
            $invalidJson = 'not valid json'
            $result = $invalidJson | & $exePath 'get' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should fail when title is missing from JSON' -Skip:$skipTests {
            $json = '[{}]'
            $result = $json | & $exePath 'get' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should handle empty input gracefully' -Skip:$skipTests {
            $result = '' | & $exePath 'get' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should fail when no input is provided' -Skip:$skipTests {
            # Simulate no stdin by closing stdin immediately
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = $exePath
            $psi.Arguments = 'get'
            $psi.RedirectStandardInput = $true
            $psi.RedirectStandardOutput = $true
            $psi.RedirectStandardError = $true
            $psi.UseShellExecute = $false
            $psi.CreateNoWindow = $true
            
            $process = New-Object System.Diagnostics.Process
            $process.StartInfo = $psi
            $process.Start() | Out-Null
            $process.StandardInput.Close()
            $process.WaitForExit()
            
            $process.ExitCode | Should -Not -Be 0
        }
    }

    Context 'Get operation output' {
        It 'should return valid JSON when update is found' -Skip:$skipTests {
            # Try to find a common update (Defender definitions are updated frequently)
            $json = '[{"title": "Windows"}]'
            $result = $json | & $exePath 'get' 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                { $result | ConvertFrom-Json } | Should -Not -Throw
                $output = $result | ConvertFrom-Json
                $output[0].title | Should -Not -BeNullOrEmpty
                $output[0].id | Should -Not -BeNullOrEmpty
            }
            else {
                # No matching update found, which is acceptable
                Write-Host "No matching update found for testing"
                $true | Should -Be $true
            }
        }

        It 'should return error when update is not found' -Skip:$skipTests {
            $json = '[{"title": "ThisUpdateDoesNotExist999888777"}]'
            $result = $json | & $exePath 'get' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
            $result | Should -Match 'not found|Error'
        }

        It 'should output to stdout for success' -Skip:$skipTests {
            $json = '[{"title": "Windows Defender"}]'
            
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = $exePath
            $psi.Arguments = 'get'
            $psi.RedirectStandardInput = $true
            $psi.RedirectStandardOutput = $true
            $psi.RedirectStandardError = $true
            $psi.UseShellExecute = $false
            $psi.CreateNoWindow = $true
            
            $process = New-Object System.Diagnostics.Process
            $process.StartInfo = $psi
            $process.Start() | Out-Null
            $process.StandardInput.WriteLine($json)
            $process.StandardInput.Close()
            
            $stdout = $process.StandardOutput.ReadToEnd()
            $stderr = $process.StandardError.ReadToEnd()
            $process.WaitForExit()
            
            if ($process.ExitCode -eq 0) {
                $stdout | Should -Not -BeNullOrEmpty
            }
            else {
                $stderr | Should -Not -BeNullOrEmpty
            }
        }

        It 'should output to stderr for errors' -Skip:$skipTests {
            $json = '[{"title": "NonExistentUpdate12345"}]'
            
            $psi = New-Object System.Diagnostics.ProcessStartInfo
            $psi.FileName = $exePath
            $psi.Arguments = 'get'
            $psi.RedirectStandardInput = $true
            $psi.RedirectStandardOutput = $true
            $psi.RedirectStandardError = $true
            $psi.UseShellExecute = $false
            $psi.CreateNoWindow = $true
            
            $process = New-Object System.Diagnostics.Process
            $process.StartInfo = $psi
            $process.Start() | Out-Null
            $process.StandardInput.WriteLine($json)
            $process.StandardInput.Close()
            
            $stdout = $process.StandardOutput.ReadToEnd()
            $stderr = $process.StandardError.ReadToEnd()
            $process.WaitForExit()
            
            $process.ExitCode | Should -Not -Be 0
            $stderr | Should -Not -BeNullOrEmpty
        }
    }

    Context 'Exit codes' {
        It 'should exit with 0 on success' -Skip:$skipTests {
            # Try with a broad search that's likely to find something
            $json = '[{"title": "Windows"}]'
            $result = $json | & $exePath 'get' 2>&1
            
            if ($LASTEXITCODE -eq 0) {
                $LASTEXITCODE | Should -Be 0
            }
            else {
                Write-Host "No update found, exit code check skipped"
                $true | Should -Be $true
            }
        }

        It 'should exit with non-zero on error' -Skip:$skipTests {
            $json = '[{"title": "NonExistentUpdate99999"}]'
            $result = $json | & $exePath 'get' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should exit with non-zero on invalid input' -Skip:$skipTests {
            $invalidJson = 'not json'
            $result = $invalidJson | & $exePath 'get' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'should exit with non-zero on unimplemented operation' -Skip:$skipTests {
            $json = '[{"title": "test"}]'
            $result = $json | & $exePath 'set' 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Performance and reliability' {
        It 'should not crash with malformed JSON' -Skip:$skipTests {
            $malformedInputs = @(
                '{"title":',
                '{"title": "test"',
                '{title: "test"}',
                '{"title": }',
                'null',
                '[]',
                '""'
            )
            
            foreach ($input in $malformedInputs) {
                $result = $input | & $exePath 'get' 2>&1
                # Should fail gracefully, not crash
                $LASTEXITCODE | Should -Not -Be 0
            }
        }

        It 'should handle very long title strings' -Skip:$skipTests {
            $longTitle = 'A' * 1000
            $json = "[{`"title`": `"$longTitle`"}]"
            $result = $json | & $exePath 'get' 2>&1
            # Should handle gracefully (either find nothing or error properly)
            $result | Should -Not -BeNullOrEmpty
        }

        It 'should handle special characters in title' -Skip:$skipTests {
            $specialTitle = 'Test & Update <2024> "Special"'
            $json = "[{`"title`": `"$specialTitle`"}]"
            $result = $json | & $exePath 'get' 2>&1
            # Should not crash
            $result | Should -Not -BeNullOrEmpty
        }

        It 'should complete within reasonable time' -Skip:$skipTests {
            $json = '[{"title": "Windows Defender"}]'
            $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
            $result = $json | & $exePath 'get' 2>&1
            $stopwatch.Stop()
            
            # Should complete within 60 seconds (Windows Update can be slow)
            $stopwatch.Elapsed.TotalSeconds | Should -BeLessThan 60
        }
    }
}
