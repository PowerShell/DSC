# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'FileContent get tests' {
    BeforeAll {
        $resourceType = 'Microsoft/FileContent'
        $filePath = Join-Path $TestDrive 'get.txt'
        [System.IO.File]::WriteAllText(
            $filePath,
            'hello',
            [System.Text.UTF8Encoding]::new($false)
        )
    }

    It 'Returns hashes without content for an existing file' {
        $json = @{ path = $filePath } | ConvertTo-Json -Compress
        $out = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actual = ($out | ConvertFrom-Json).actualState

        $actual.path | Should -BeExactly $filePath
        $actual._exist | Should -BeTrue
        $actual.sha256 | Should -BeExactly '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
        $actual.sha512 | Should -BeExactly '9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043'
        $actual.PSObject.Properties.Name | Should -Not -Contain 'content'
    }

    It 'Returns nonexistence without hashes for a missing file' {
        $missingPath = Join-Path $TestDrive 'missing.txt'
        $json = @{ path = $missingPath } | ConvertTo-Json -Compress
        $out = $json | dsc resource get -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actual = ($out | ConvertFrom-Json).actualState

        $actual._exist | Should -BeFalse
        $actual.PSObject.Properties.Name | Should -Not -Contain 'sha256'
        $actual.PSObject.Properties.Name | Should -Not -Contain 'sha512'
    }

    It 'Accepts the short input argument' {
        $json = @{ path = $filePath } | ConvertTo-Json -Compress
        $actual = filecontent get -i $json 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        $actual._exist | Should -BeTrue
    }

    It 'Reports a missing input value' {
        $null = filecontent get --input 2>$TestDrive/error.log

        $LASTEXITCODE | Should -Be 1
        (Get-Content -Raw $TestDrive/error.log) | Should -Match 'Missing value for --input argument'
    }
}
