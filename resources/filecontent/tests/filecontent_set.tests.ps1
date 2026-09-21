# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'FileContent set tests' {
    BeforeAll {
        $resourceType = 'Microsoft.Filesystem.File/Content'
    }

    BeforeEach {
        $testRoot = Join-Path $TestDrive "$([System.Guid]::NewGuid())"
        $null = New-Item -ItemType Directory -Path $testRoot
        $filePath = Join-Path $testRoot 'file.txt'
    }

    AfterEach {
        Remove-Item -LiteralPath $testRoot -Recurse -Force -ErrorAction Ignore
    }

    It 'Creates a UTF-8 file and returns its hashes' {
        $json = @{ path = $filePath; content = 'hello' } | ConvertTo-Json -Compress
        $out = $json | dsc resource set -r $resourceType -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actual = ($out | ConvertFrom-Json).afterState

        [System.IO.File]::ReadAllText($filePath) | Should -BeExactly 'hello'
        $actual.content | Should -BeExactly 'hello'
        $actual._exist | Should -BeTrue
        $actual.sha256 | Should -BeExactly '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'
    }

    It 'Creates missing parent directories and reports the creation when _exist is <existSetting>' -ForEach @(
        @{ existSetting = 'true'; includeExist = $true }
        @{ existSetting = 'omitted'; includeExist = $false }
    ) {
        $nestedDirectory = Join-Path (Join-Path $testRoot 'first') 'second'
        $nestedFilePath = Join-Path $nestedDirectory 'file.txt'
        $stderrPath = Join-Path $testRoot 'stderr.log'
        $inputState = @{ path = $nestedFilePath; content = 'nested' }
        if ($includeExist) {
            $inputState._exist = $true
        }
        $json = $inputState | ConvertTo-Json -Compress

        $out = $json | dsc -l info resource set -r $resourceType -f - 2>$stderrPath
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $stderrPath)
        $actual = ($out | ConvertFrom-Json).afterState

        [System.IO.File]::ReadAllText($nestedFilePath) | Should -BeExactly 'nested'
        $actual._exist | Should -BeTrue
        (Get-Content -Raw $stderrPath) |
            Should -BeLike "*INFO*Creating parent directory '$nestedDirectory'*"
    }

    It 'Reports an error when a file blocks parent directory creation' {
        $blockingPath = Join-Path $testRoot 'blocked'
        [System.IO.File]::WriteAllText($blockingPath, 'blocking file')
        $blockedParent = Join-Path $blockingPath 'child'
        $blockedFilePath = Join-Path $blockedParent 'file.txt'
        $stderrPath = Join-Path $testRoot 'stderr.log'
        $json = @{ path = $blockedFilePath; content = 'blocked' } | ConvertTo-Json -Compress

        $null = $json | dsc resource set -r $resourceType -f - 2>$stderrPath

        $LASTEXITCODE | Should -Not -Be 0
        (Get-Content -Raw $stderrPath) |
            Should -BeLike "*Failed to create parent directory '$blockedParent'*"
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
