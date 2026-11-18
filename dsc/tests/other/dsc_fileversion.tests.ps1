# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows executable version resources' -Skip:(-not $IsWindows) {
    It 'has nonempty ProductVersion and FileVersion for all built executables' {
        $binDebug = "$PSScriptRoot\..\..\..\bin\debug"
        $binRelease = "$PSScriptRoot\..\..\..\bin\release"
        if (Test-Path $binDebug) {
            $executables = @(Get-ChildItem -Path $binDebug -Filter '*.exe' -File -ErrorAction Stop)
        } elseif (Test-Path $binRelease) {
            $executables = @(Get-ChildItem -Path $binRelease -Filter '*.exe' -File -ErrorAction Stop)
        } else {
            throw 'Binaries not found from build'
        }

        $executables | Should -Not -BeNullOrEmpty

        foreach ($executable in $executables) {
            $executable.VersionInfo.ProductVersion | Should -Not -BeNullOrEmpty -Because "$($executable.Name) must have a ProductVersion"
            $executable.VersionInfo.FileVersion | Should -Not -BeNullOrEmpty -Because "$($executable.Name) must have a FileVersion"
        }
    }
}
