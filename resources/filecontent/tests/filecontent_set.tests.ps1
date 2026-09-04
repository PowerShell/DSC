# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'FileContent set tests' {
    BeforeAll {
        $resourceType = 'Microsoft/FileContent'
    }

    BeforeEach {
        $filePath = Join-Path $TestDrive "$([System.Guid]::NewGuid()).txt"
    }

    AfterEach {
        Remove-Item -LiteralPath $filePath -Force -ErrorAction Ignore
    }

    It 'Creates a UTF-8 file and returns its hashes' {
        $json = @{ path = $filePath; content = 'hello' } | ConvertTo-Json -Compress
        $out = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actual = ($out | ConvertFrom-Json).afterState

        [System.IO.File]::ReadAllText($filePath) | Should -BeExactly 'hello'
        $actual._exist | Should -BeTrue
        $actual.sha256 | Should -BeExactly '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
        $actual.PSObject.Properties.Name | Should -Not -Contain 'content'
    }

    It 'Removes a file when _exist is false' {
        [System.IO.File]::WriteAllText($filePath, 'remove me')
        $json = @{ path = $filePath; _exist = $false } | ConvertTo-Json -Compress
        $out = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actual = ($out | ConvertFrom-Json).afterState

        $filePath | Should -Not -Exist
        $actual._exist | Should -BeFalse
    }
}
