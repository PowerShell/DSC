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

    It 'Creates a file from configuration with multiline content starting with an escaped bracket' {
        $configYaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Certificate authority policy
    type: $resourceType
    properties:
      path: '$filePath'
      content: |
        [[Version]
        Signature="`$Windows NT`$"

        [PolicyStatementExtension]
        Policies=InternalPolicy

        [InternalPolicy]
        OID=1.2.3.4.1455.67.89.5
        Notice="Legal Policy Statement"
        URL=https://pki.corp.contoso.com/pki/cps.txt
"@
        $expectedContent = @(
            '[Version]'
            'Signature="$Windows NT$"'
            ''
            '[PolicyStatementExtension]'
            'Policies=InternalPolicy'
            ''
            '[InternalPolicy]'
            'OID=1.2.3.4.1455.67.89.5'
            'Notice="Legal Policy Statement"'
            'URL=https://pki.corp.contoso.com/pki/cps.txt'
            ''
        ) -join "`n"

        $out = $configYaml | dsc config set -f - 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result = $out | ConvertFrom-Json

        $result.hadErrors | Should -BeFalse
        [System.IO.File]::ReadAllText($filePath) | Should -BeExactly $expectedContent
    }

    It 'Creates missing parent directories and reports the creation when _exist is <existSetting>' -ForEach @(
        @{ existSetting = 'true'; includeExist = $true }
        @{ existSetting = 'omitted'; includeExist = $false }
    ) {
        $existingDirectory = Join-Path $testRoot 'a'
        $null = New-Item -ItemType Directory -Path $existingDirectory
        $firstMissingDirectory = Join-Path $existingDirectory 'b'
        $secondMissingDirectory = Join-Path $firstMissingDirectory 'c'
        $nestedFilePath = Join-Path $secondMissingDirectory 'file.txt'
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
        $firstMissingDirectory | Should -Exist
        $secondMissingDirectory | Should -Exist
        $actual._exist | Should -BeTrue
        (Get-Content -Raw $stderrPath) |
            Should -BeLike "*INFO*Creating parent directory '$secondMissingDirectory'*"
    }

    It 'Reports an error when an intermediate parent path is a file' {
        $existingDirectory = Join-Path $testRoot 'a'
        $null = New-Item -ItemType Directory -Path $existingDirectory
        $blockingPath = Join-Path $existingDirectory 'b'
        [System.IO.File]::WriteAllText($blockingPath, 'blocking file')
        $blockedParent = Join-Path $blockingPath 'c'
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
