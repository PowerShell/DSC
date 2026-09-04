# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'FileContent export tests' {
    BeforeAll {
        $resourceType = 'Microsoft/FileContent'
        $filePath = Join-Path $TestDrive 'export.txt'
        [System.IO.File]::WriteAllText(
            $filePath,
            "hello`nworld",
            [System.Text.UTF8Encoding]::new($false)
        )
    }

    It 'Returns content and hashes' {
        $json = @{ path = $filePath } | ConvertTo-Json -Compress
        $out = $json | dsc resource export -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $properties = ($out | ConvertFrom-Json).resources[0].properties

        $properties.path | Should -BeExactly $filePath
        $properties.content | Should -BeExactly "hello`nworld"
        $properties.sha256 | Should -Not -BeNullOrEmpty
        $properties.sha512 | Should -Not -BeNullOrEmpty
        $properties._exist | Should -BeTrue
    }
}
