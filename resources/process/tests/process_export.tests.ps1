# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.System/Process export' {
    BeforeAll {
        $children = @(
            1..6 | ForEach-Object {
                $startInfo = [System.Diagnostics.ProcessStartInfo]::new((Get-Command dsctest).Source)
                $startInfo.UseShellExecute = $false
                $startInfo.ArgumentList.Add('sleep')
                $startInfo.ArgumentList.Add('--input')
                $startInfo.ArgumentList.Add('{"seconds":120}')
                [System.Diagnostics.Process]::Start($startInfo)
            }
        )
    }

    AfterAll {
        foreach ($child in $children) {
            if (-not $child.HasExited) {
                Stop-Process -Id $child.Id
            }
            $child.Dispose()
        }
    }

    It 'Exports all processes without the old four-process limit' {
        $out = dsc resource export -r Microsoft.System/Process 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $instances = ($out | ConvertFrom-Json).resources.properties
        foreach ($child in $children) {
            $instances.pid | Should -Contain $child.Id
        }
    }

    It 'Filters export by PID' {
        $json = @{ pid = $PID } | ConvertTo-Json -Compress
        $out = $json | dsc resource export -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $instances = ($out | ConvertFrom-Json).resources
        $instances | Should -HaveCount 1
        $instances[0].properties.pid | Should -Be $PID
    }

    It 'Filters by exact command-line arguments' {
        $query = @{ pid = $PID } | ConvertTo-Json -Compress
        $out = $query | dsc resource get -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $state = ($out | ConvertFrom-Json).actualState
        $json = @{ pid = $PID; cmdline = @($state.cmdline) } | ConvertTo-Json -Compress
        $out = $json | dsc resource export -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $instances = ($out | ConvertFrom-Json).resources
        $instances | Should -HaveCount 1
        $instances[0].properties.cmdline | Should -Be $state.cmdline
    }

    It 'Rejects invalid export filters' {
        '{"name":""}' | dsc resource export -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive\error.log | Should -Not -BeNullOrEmpty
    }

    It 'Supports name filter <Pattern> and combines it with PID' -ForEach @(
        @{ Pattern = '*' }
        @{ Pattern = '*w*h*' }
        @{ Pattern = 'pw*' }
        @{ Pattern = '*sh*' }
    ) {
        $json = @{ pid = $PID; name = $Pattern } | ConvertTo-Json -Compress
        $out = $json | dsc resource export -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $instances = ($out | ConvertFrom-Json).resources
        $instances | Should -HaveCount 1
        $instances[0].properties.pid | Should -Be $PID
    }

    It 'Returns no instances successfully for unmatched filters' {
        $json = @{ pid = $PID; name = 'dsc-no-such-process-cf04a8a2' } | ConvertTo-Json -Compress
        $out = $json | dsc resource export -r Microsoft.System/Process -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        @((($out | ConvertFrom-Json).resources)) | Should -HaveCount 0
    }

    It 'Supports filtering through configuration export' {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(@{
                name = 'Current process'
                type = 'Microsoft.System/Process'
                properties = @{ pid = $PID; name = '*' }
            })
        } | ConvertTo-Json -Depth 5
        $out = $config | dsc config export -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $instances = ($out | ConvertFrom-Json).resources
        $instances | Should -HaveCount 1
        $instances[0].properties.pid | Should -Be $PID
    }
}
