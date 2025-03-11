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
        $resultJson.hadErrors | Should -BeFalse
        $path = $resultJson.results.result.afterState.path
        $name = $resultJson.results.result.afterState.name

        $path | Should -Be $env:TEMP
        (Join-Path $path $name) | Should -Exist
        Get-Item $resultJson.results.result.afterState.path | Should -BeOfType 'System.IO.FileInfo'
    }

    It 'Filesys resource can create directory' {
        $resultJson = dsc config set -f "../examples/filesys_dir_create.dsc.yaml" | ConvertFrom-Json
        $resultJson.hadErrors | Should -BeFalse
        $resultJson.results.result.afterState.path | Should -Exist
        Get-Item $resultJson.results.result.afterState.path | Should -BeOfType 'System.IO.DirectoryInfo'

    }

    It 'Filesys resource can create file with content' {
        $resultJson = dsc config set -f "../examples/filesys_filecontent.dsc.yaml" | ConvertFrom-Json
        $resultJson.hadErrors | Should -BeFalse

        $resultFilePath = $resultJson.results.result.afterState.path
        $resultFilePath | Should -Exist
        Get-Content $resultFilePath | Should -Be "Hello, World!"
    }

    It 'Filesys resource can delete a file' {
        if (-not (Test-Path $testFile)) {
            New-Item -Path $testFile -ItemType File -Force | Out-Null
        }

        $resultJson = dsc config set -f "../examples/filesys_delete.dsc.yaml" | ConvertFrom-Json
        $resultJson.hadErrors | Should -BeFalse
        $resultFilePath = $resultJson.results.result.afterState.path
        $resultFilePath | Should -Not -Exist
    }

    It 'Filesys resource can delete an empty directory' -Pending {
        if (-not (Test-Path $testDir)) {
            New-Item -Path $testDir -ItemType Directory -Force | Out-Null
        }

        $resultJson = dsc config set -f "../examples/filesys_dir_delete.dsc.yaml" | ConvertFrom-Json
        $resultJson.hadErrors | Should -BeFalse
        $resultDirPath = $resultJson.results.result.afterState.path
        $resultDirPath | Should -Not -Exist
    }

    It 'Filesys resource can delete a non-empty directory' -Pending {
        if (-not (Test-Path $testDir)) {
            New-Item -Path $testDir -ItemType Directory -Force | Out-Null
            New-Item -Path (Join-Path $testDir $testFileName) -ItemType File -Force | Out-Null
        }

        $resultJson = dsc config set -f "../examples/filesys_dir_delete.dsc.yaml" | ConvertFrom-Json
        $resultJson.hadErrors | Should -BeFalse
        $resultDirPath = $resultJson.results.result.afterState.path
        $resultDirPath | Should -Not -Exist
    }

    It 'Filesys resource can delete a directory recursively' -Pending {
        if (-not (Test-Path $testDir)) {
            $dirPath = New-Item -Path $testDir -ItemType Directory -Force | Out-Null
            $subDirPath = New-Item -Path (Join-Path $dirPath 'test-subdir') -ItemType Directory -Force | Out-Null
            New-Item -Path (Join-Path $subDirPath $testFileName) -ItemType File -Force | Out-Null
        }

        $resultJson = dsc config set -f "../examples/filesys_dir_delete_recursive.dsc.yaml" | ConvertFrom-Json
        $resultJson.hadErrors | Should -BeFalse
        $resultDirPath = $resultJson.results.result.afterState.path
        $resultDirPath | Should -Not -Exist
    }
}