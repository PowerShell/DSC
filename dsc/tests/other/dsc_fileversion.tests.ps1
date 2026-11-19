# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows executable version resources' -Skip:(-not $IsWindows) {
    BeforeAll {
        $dscFolder = Split-Path (Get-Command dsc).Source -Parent
        $executables = @(Get-ChildItem -Path $dscFolder -Filter '*.exe' -File -ErrorAction Stop)
    }

    It 'has nonempty ProductVersion and FileVersion for all built executables' {
        $executables | Should -Not -BeNullOrEmpty

        foreach ($executable in $executables) {
            $executable.VersionInfo.ProductVersion | Should -Not -BeNullOrEmpty -Because "$($executable.Name) must have a ProductVersion"
            $executable.VersionInfo.FileVersion | Should -Not -BeNullOrEmpty -Because "$($executable.Name) must have a FileVersion"
        }
    }
}
