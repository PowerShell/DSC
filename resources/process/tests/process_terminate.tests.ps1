# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.System.Action/TerminateProcess' {
    BeforeAll {
        $extension = if ($IsWindows) { '.exe' } else { '' }
        $testExecutable = Join-Path $TestDrive ("dp" + [guid]::NewGuid().ToString('N').Substring(0, 10) + $extension)
        Copy-Item -LiteralPath (Get-Command dsctest).Source -Destination $testExecutable

        function Start-TestProcess {
            $startInfo = [System.Diagnostics.ProcessStartInfo]::new($testExecutable)
            $startInfo.UseShellExecute = $false
            $startInfo.ArgumentList.Add('sleep')
            $startInfo.ArgumentList.Add('--input')
            $startInfo.ArgumentList.Add('{"seconds":120}')
            [System.Diagnostics.Process]::Start($startInfo)
        }

        function Invoke-TerminateAction {
            param([hashtable]$Properties)

            $config = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
                resources = @(@{
                    name = 'Terminate test processes'
                    type = 'Microsoft.System.Action/TerminateProcess'
                    properties = $Properties
                })
            } | ConvertTo-Json -Depth 5
            $config | dsc config set -f - 2>$TestDrive\error.log
        }
    }

    BeforeEach {
        $children = [System.Collections.Generic.List[System.Diagnostics.Process]]::new()
    }

    AfterEach {
        foreach ($child in $children) {
            if (-not $child.HasExited) {
                Stop-Process -Id $child.Id
            }
            $child.Dispose()
        }
    }

    It 'Is discoverable with only the set operation' {
        $out = dsc action list Microsoft.System.Action/TerminateProcess | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.supportedOperations | Should -Be @('set')
        $out.inputSchema | Should -Not -BeNullOrEmpty
        $out.outputSchema | Should -Not -BeNullOrEmpty
    }

    It 'Terminates only the selected PID' {
        1..2 | ForEach-Object {
            $children.Add((Start-TestProcess))
        }
        $out = Invoke-TerminateAction -Properties @{ pid = $children[0].Id }
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $children[0].WaitForExit(5000) | Should -BeTrue
        $children[1].HasExited | Should -BeFalse
        $terminated = ($out | ConvertFrom-Json).results[0].result.afterState.terminated
        $terminated | Should -HaveCount 1
        $terminated[0].pid | Should -Be $children[0].Id
    }

    It 'Gets a process by exact name and reports ambiguous matches' {
        $children.Add((Start-TestProcess))
        $query = @{ pid = $children[0].Id } | ConvertTo-Json -Compress
        $out = $query | dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $query = @{ name = ($out | ConvertFrom-Json).actualState.name } | ConvertTo-Json -Compress
        $out = $query | dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        ($out | ConvertFrom-Json).actualState.pid | Should -Be $children[0].Id
        $children.Add((Start-TestProcess))
        $query | dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive\error.log | Should -Match 'More than one process'
    }

    It 'Terminates every process with the exact name' {
        1..3 | ForEach-Object {
            $children.Add((Start-TestProcess))
        }
        $query = @{ pid = $children[0].Id } | ConvertTo-Json -Compress
        $state = $query | dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $name = ($state | ConvertFrom-Json).actualState.name
        $out = Invoke-TerminateAction -Properties @{ name = $name }
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        foreach ($child in $children) {
            $child.WaitForExit(5000) | Should -BeTrue
        }
        $terminated = ($out | ConvertFrom-Json).results[0].result.afterState.terminated
        $terminated | Should -HaveCount 3
        foreach ($child in $children) {
            $terminated.pid | Should -Contain $child.Id
        }
    }

    It 'Rejects invalid or missing targets <Properties>' -ForEach @(
        @{ Properties = @{} }
        @{ Properties = @{ name = '' } }
        @{ Properties = @{ name = '*' } }
        @{ Properties = @{ pid = 0 } }
        @{ Properties = @{ pid = 1; name = 'not-a-process' } }
        @{ Properties = @{ name = 'dsc-no-such-process-cf04a8a2' } }
    ) {
        Invoke-TerminateAction -Properties $Properties
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive\error.log | Should -Not -BeNullOrEmpty
    }
}
