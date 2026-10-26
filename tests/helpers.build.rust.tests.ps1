# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force
}

Describe 'Rust build and test project selection' {
    InModuleScope helpers.build {
        BeforeEach {
            Mock Set-DefaultWorkspaceMemberGroup {}
            Mock Reset-DefaultWorkspaceMemberGroup {}
            Mock Get-DefaultWorkspaceMemberGroup { 'workspace defaults' }
            Mock Test-ShouldSkipProject { $false }
            Mock Test-ShouldSkipProjectTesting { $false }
            Mock cargo { $global:LASTEXITCODE = 0 }
            $selected = [DscProjectDefinition]@{
                Name = 'selected'
                RustPackageName = 'selected-package'
                IsRust = $true
            }
            $nonRust = [DscProjectDefinition]@{ Name = 'script' }
        }

        It 'Builds only the selected Rust packages and honors package aliases' {
            $another = [DscProjectDefinition]@{ Name = 'another'; IsRust = $true }
            Build-RustProject -Project @($selected, $another, $nonRust)

            Should -Invoke cargo -Exactly 1 -ParameterFilter {
                $args -contains 'build' -and $args -contains '-p' -and
                $args -contains 'selected-package' -and $args -contains 'another' -and
                @($args | Where-Object { $_ -eq '-p' }).Count -eq 2 -and
                $args -notcontains 'script'
            }
        }

        It 'Tests only the selected packages and preserves the test filter' {
            Test-RustProject -Project @($selected, $nonRust) -TestFilter 'specific_test'

            Should -Invoke cargo -Exactly 1 -ParameterFilter {
                $args -contains 'test' -and $args -contains 'selected-package' -and
                $args[-1] -eq 'specific_test'
            }
        }

        It 'Does not fall back to the workspace for non-Rust selections' {
            Build-RustProject -Project $nonRust
            Test-RustProject -Project $nonRust

            Should -Invoke cargo -Exactly 0
        }

        It 'Excludes projects that disable Rust tests' {
            $selected.SkipTestProject = $true

            Test-RustProject -Project $selected

            Should -Invoke cargo -Exactly 0
        }

        It 'Preserves workspace defaults when no project is supplied' {
            Build-RustProject
            Test-RustProject

            Should -Invoke cargo -Exactly 2 -ParameterFilter { $args -notcontains '-p' }
        }

        It 'Excludes projects unsupported on the current platform' {
            Mock Test-ShouldSkipProject { $true }

            Build-RustProject -Project $selected
            Test-RustProject -Project $selected

            Should -Invoke cargo -Exactly 0
        }
    }
}
