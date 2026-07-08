# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'ConvertTo-NormalizedLcovSourcePath' {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force
    }

    Context 'Linux GitHub Actions paths' {
        It 'Strips /home/runner/work/<repo>/<repo>/ prefix' {
            $result = ConvertTo-NormalizedLcovSourcePath -RawPath '/home/runner/work/DSC/DSC/dsc_lib/src/dscresources/command_resource.rs'
            $result | Should -Be 'dsc_lib/src/dscresources/command_resource.rs'
        }

        It 'Handles varying usernames' {
            $result = ConvertTo-NormalizedLcovSourcePath -RawPath '/home/vsts/work/MyProject/MyProject/src/main.rs'
            $result | Should -Be 'src/main.rs'
        }
    }

    Context 'macOS GitHub Actions paths' {
        It 'Strips /Users/runner/work/<repo>/<repo>/ prefix' {
            $result = ConvertTo-NormalizedLcovSourcePath -RawPath '/Users/runner/work/DSC/DSC/dsc_lib/src/dscresources/command_resource.rs'
            $result | Should -Be 'dsc_lib/src/dscresources/command_resource.rs'
        }
    }

    Context 'Windows GitHub Actions paths' {
        It 'Strips D:/a/<repo>/<repo>/ prefix after backslash normalization' {
            $result = ConvertTo-NormalizedLcovSourcePath -RawPath 'D:\a\DSC\DSC\dsc_lib\src\dscresources\command_resource.rs'
            $result | Should -Be 'dsc_lib/src/dscresources/command_resource.rs'
        }

        It 'Handles C: drive' {
            $result = ConvertTo-NormalizedLcovSourcePath -RawPath 'C:\a\DSC\DSC\dsc_lib\src\main.rs'
            $result | Should -Be 'dsc_lib/src/main.rs'
        }
    }

    Context 'Already relative paths' {
        It 'Returns relative paths unchanged (with normalized separators)' {
            $result = ConvertTo-NormalizedLcovSourcePath -RawPath 'dsc_lib/src/main.rs'
            $result | Should -Be 'dsc_lib/src/main.rs'
        }

        It 'Normalizes backslashes in relative paths' {
            $result = ConvertTo-NormalizedLcovSourcePath -RawPath 'dsc_lib\src\main.rs'
            $result | Should -Be 'dsc_lib/src/main.rs'
        }
    }

    Context 'Cross-platform consistency' {
        It 'Produces the same output for the same file on all platforms' {
            $linuxPath = '/home/runner/work/DSC/DSC/dsc_lib/src/configure.rs'
            $macosPath = '/Users/runner/work/DSC/DSC/dsc_lib/src/configure.rs'
            $windowsPath = 'D:\a\DSC\DSC\dsc_lib\src\configure.rs'

            $linuxResult = ConvertTo-NormalizedLcovSourcePath -RawPath $linuxPath
            $macosResult = ConvertTo-NormalizedLcovSourcePath -RawPath $macosPath
            $windowsResult = ConvertTo-NormalizedLcovSourcePath -RawPath $windowsPath

            $linuxResult | Should -Be $macosResult
            $macosResult | Should -Be $windowsResult
            $linuxResult | Should -Be 'dsc_lib/src/configure.rs'
        }
    }
}

Describe 'Merge-LcovFile' {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force
    }

    Context 'Merging files with different platform paths for the same source' {
        BeforeAll {
            $linuxLcov = Join-Path $TestDrive 'linux-lcov.info'
            $macosLcov = Join-Path $TestDrive 'macos-lcov.info'
            $windowsLcov = Join-Path $TestDrive 'windows-lcov.info'
            $outputPath = Join-Path $TestDrive 'merged.info'

            # Same file, same line coverage, different absolute paths
            @'
SF:/home/runner/work/DSC/DSC/dsc_lib/src/main.rs
DA:1,5
DA:2,3
DA:3,0
LF:3
LH:2
end_of_record
'@ | Set-Content -Path $linuxLcov -NoNewline

            @'
SF:/Users/runner/work/DSC/DSC/dsc_lib/src/main.rs
DA:1,2
DA:2,0
DA:3,4
LF:3
LH:2
end_of_record
'@ | Set-Content -Path $macosLcov -NoNewline

            @'
SF:D:\a\DSC\DSC\dsc_lib\src\main.rs
DA:1,1
DA:2,0
DA:3,0
LF:3
LH:1
end_of_record
'@ | Set-Content -Path $windowsLcov -NoNewline

            Merge-LcovFile -Path @($linuxLcov, $macosLcov, $windowsLcov) -OutputPath $outputPath
            $script:mergedContent = Get-Content -Path $outputPath -Raw
        }

        It 'Produces a single source file entry (not three duplicates)' {
            $sfCount = ([regex]::Matches($script:mergedContent, '^SF:', [System.Text.RegularExpressions.RegexOptions]::Multiline)).Count
            $sfCount | Should -Be 1
        }

        It 'Sums hit counts for matching lines' {
            # Line 1: 5 + 2 + 1 = 8
            $script:mergedContent | Should -Match '(?m)^DA:1,8$'
            # Line 2: 3 + 0 + 0 = 3
            $script:mergedContent | Should -Match '(?m)^DA:2,3$'
            # Line 3: 0 + 4 + 0 = 4
            $script:mergedContent | Should -Match '(?m)^DA:3,4$'
        }

        It 'Reports all 3 lines covered (all have non-zero sum)' {
            $script:mergedContent | Should -Match '(?m)^LH:3$'
        }

        It 'Reports 3 total lines' {
            $script:mergedContent | Should -Match '(?m)^LF:3$'
        }
    }

    Context 'Merging files with unique sources that do not overlap' {
        BeforeAll {
            $lcov1 = Join-Path $TestDrive 'lcov1.info'
            $lcov2 = Join-Path $TestDrive 'lcov2.info'
            $outputPath = Join-Path $TestDrive 'merged-unique.info'

            @'
SF:/home/runner/work/DSC/DSC/dsc_lib/src/foo.rs
DA:1,5
LF:1
LH:1
end_of_record
'@ | Set-Content -Path $lcov1 -NoNewline

            @'
SF:/home/runner/work/DSC/DSC/dsc_lib/src/bar.rs
DA:1,3
LF:1
LH:1
end_of_record
'@ | Set-Content -Path $lcov2 -NoNewline

            Merge-LcovFile -Path @($lcov1, $lcov2) -OutputPath $outputPath
            $script:mergedUniqueContent = Get-Content -Path $outputPath -Raw
        }

        It 'Preserves both source file entries' {
            $sfCount = ([regex]::Matches($script:mergedUniqueContent, '^SF:', [System.Text.RegularExpressions.RegexOptions]::Multiline)).Count
            $sfCount | Should -Be 2
        }
    }
}
