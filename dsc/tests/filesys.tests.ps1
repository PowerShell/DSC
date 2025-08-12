# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'FileSys resoure tests' {
    BeforeAll {
        $testDir = Join-Path $env:TEMP 'test-dir-resource'
        $testFile = Join-Path $testDir 'test-file-resource.txt'
        $testFileName = 'test-file-resource.txt'
    }

    It 'Filesys resource can create file' {
        if (Test-Path $testFile) {
            Remove-Item -Path $testFile -Force
        }

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_create.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse
        $path = $resultJson.results.result.afterState.path
        $path | Should -Exist
        Get-Item $resultJson.results.result.afterState.path | Should -BeOfType 'System.IO.FileInfo'
    }

    It 'Filesys resource can create directory' {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Force -Recurse
        }

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_dir_create.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse
        $resultJson.results.result.afterState.path | Should -Exist
        Get-Item $resultJson.results.result.afterState.path | Should -BeOfType 'System.IO.DirectoryInfo'
    }

    It 'Filesys resource can create file with content' {
        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_filecontent.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse

        $resultFilePath = $resultJson.results.result.afterState.path
        $resultFilePath | Should -Exist
        Get-Content $resultFilePath | Should -Be "Hello, World!"
    }

    It 'Filesys resource can delete a file' {
        if (-not (Test-Path $testFile)) {
            New-Item -Path $testFile -ItemType File -Force | Out-Null
        }

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_delete.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse
        $resultFilePath = $resultJson.results.result.afterState.path
        $resultFilePath | Should -Not -Exist
    }

    It 'Filesys resource can delete an empty directory' {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Force -Recurse
        }

        New-Item -Path $testDir -ItemType Directory -Force | Out-Null

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_dir_delete.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse
        $resultDirPath = $resultJson.results.result.afterState.path
        $resultDirPath | Should -Not -Exist
    }

    It 'Filesys resource cannot delete a non-empty directory' {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Force -Recurse
        }

        New-Item -Path $testDir -ItemType Directory -Force | Out-Null
        New-Item -Path (Join-Path $testDir $testFileName) -ItemType File -Force | Out-Null

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_dir_delete.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Not -Be 0
        $testDir | Should -Exist
    }

    It 'Filesys resource can delete a directory recursively' {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Force -Recurse
        }

        $dirPath = New-Item -Path $testDir -ItemType Directory -Force
        $subDirPath = New-Item -Path (Join-Path $dirPath 'test-subdir') -ItemType Directory -Force
        New-Item -Path (Join-Path $subDirPath $testFileName) -ItemType File -Force | Out-Null

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_dir_delete_recurse.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse
        $resultDirPath = $resultJson.results.result.afterState.path
        $resultDirPath | Should -Not -Exist
    }

    It 'Can create file if parent directory does not exist' {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Force -Recurse
        }

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_create_parent.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse
        $resultJson.results.result.afterState.path | Should -Exist
        Get-Item $resultJson.results.result.afterState.path | Should -BeOfType 'System.IO.FileInfo'
    }

    It 'Can create file with content if parent directory does not exist' {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Force -Recurse
        }

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_filecontent_parent.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse

        $resultFilePath = $resultJson.results.result.afterState.path
        $resultFilePath | Should -Exist
        Get-Content $resultFilePath | Should -Be "Hello, World!"
    }

    It 'Can create directory if parent directory does not exist' {
        if (Test-Path $testDir) {
            Remove-Item -Path $testDir -Force -Recurse
        }

        $resultJson = dsc config set -f "$PSScriptRoot/../examples/filesys_dir_create_parent.dsc.yaml" | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resultJson.hadErrors | Should -BeFalse
        $resultJson.results.result.afterState.path | Should -Exist
        Get-Item $resultJson.results.result.afterState.path | Should -BeOfType 'System.IO.DirectoryInfo'
    }
}