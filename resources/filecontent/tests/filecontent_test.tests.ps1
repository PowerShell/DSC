# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'FileContent test tests' {
    BeforeAll {
        $resourceType = 'Microsoft/FileContent'
        $filePath = Join-Path $TestDrive 'test.txt'
        [System.IO.File]::WriteAllText(
            $filePath,
            'hello',
            [System.Text.UTF8Encoding]::new($false)
        )
    }

    It 'Compares content to the file by hash when content is the only desired value' {
        $json = @{ path = $filePath; content = 'hello' } | ConvertTo-Json -Compress
        $out = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Detects different content when content is the only desired value' {
        $json = @{ path = $filePath; content = 'different' } | ConvertTo-Json -Compress
        $out = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeFalse
    }

    It 'Compares supplied SHA-256 and SHA-512 hashes' {
        $json = @{
            path = $filePath
            sha256 = '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
            sha512 = '9b71d224bd62f3785d96d46ad3ea3d73319bfbc2890caadae2dff72519673ca72323c3d99ba5c11d7c7acc6e14b8c5da0c4663475c2e5c3adef46f73bcdec043'
        } | ConvertTo-Json -Compress
        $out = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Reports desired state when an absent file should not exist' {
        $missingPath = Join-Path $TestDrive 'absent.txt'
        $json = @{ path = $missingPath; _exist = $false } | ConvertTo-Json -Compress
        $out = $json | dsc resource test -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }
}
